// ----------------------------------------------------
// Structs & Uniforms
// ----------------------------------------------------

struct CameraUniform {
    position: vec3<f32>,
    flen: f32,
    camera_to_world: mat4x4<f32>,
}

struct Material {
    material_type: u32, // 0: Diffuse, 1: Specular, 2: Emissive
    pad0: u32,
    pad1: u32,
    pad2: u32,
    color: vec3<f32>,
    pad3: f32,
}

struct Sphere {
    position: vec3<f32>,
    radius: f32,
    material: Material,
}

struct ConfigUniform {
    environment: Material,
    frame_count: u32,
    sphere_count: u32,
    width: u32,
    height: u32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct Hit {
    distance: f32,
    position: vec3<f32>,
    normal: vec3<f32>,
    material_type: u32,
    material_color: vec3<f32>,
    material_ior: f32,
}

// Bindings for Compute Shader
@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<storage, read> spheres: array<Sphere>;
@group(0) @binding(2) var<uniform> config: ConfigUniform;
@group(0) @binding(3) var<storage, read_write> accumulation_buffer: array<vec4<f32>>;
@group(0) @binding(4) var output_texture: texture_storage_2d<rgba8unorm, write>;

// ----------------------------------------------------
// Random Utility Functions (PCG 3D)
// ----------------------------------------------------

fn pcg3d(p: ptr<function, vec3<u32>>) -> vec3<u32> {
    *p = *p * 1664525u + 1013904223u;
    (*p).x += (*p).y * (*p).z;
    (*p).y += (*p).z * (*p).x;
    (*p).z += (*p).x * (*p).y;
    *p = *p ^ (*p >> vec3<u32>(16u));
    (*p).x += (*p).y * (*p).z;
    (*p).y += (*p).z * (*p).x;
    (*p).z += (*p).x * (*p).y;
    return *p;
}

fn rng_next_f32(state: ptr<function, vec3<u32>>) -> f32 {
    let r = pcg3d(state);
    return f32(r.x) * (1.0 / 4294967295.0);
}

// ----------------------------------------------------
// Raytracing Helper Functions
// ----------------------------------------------------

const PI: f32 = 3.14159265359;

fn tangentspace_basis(n: vec3<f32>) -> array<vec3<f32>, 2> {
    var sg = 1.0;
    if (n.z < 0.0) {
        sg = -1.0;
    }
    let a_factor = -1.0 / (sg + n.z);
    let b_factor = n.x * n.y * a_factor;
    let t = vec3<f32>(1.0 + sg * n.x * n.x * a_factor, sg * b_factor, -sg * n.x);
    let b = vec3<f32>(b_factor, sg + n.y * n.y * a_factor, -n.y);
    return array<vec3<f32>, 2>(t, b);
}

fn sample_hemisphere_cosine(u1: f32, u2: f32) -> vec3<f32> {
    let r = sqrt(u1);
    let theta = u2 * 2.0 * PI;
    return vec3<f32>(r * cos(theta), r * sin(theta), sqrt(1.0 - u1));
}

fn sphere_distance(sphere: Sphere, ray_origin: vec3<f32>, ray_direction: vec3<f32>) -> f32 {
    let position = ray_origin - sphere.position;
    let b = dot(ray_direction, position);
    let c = dot(position, position) - sphere.radius * sphere.radius;
    if (c < b * b) {
        var t = b + sqrt(b * b - c);
        if (0.0 < t) {
            t = b - sqrt(b * b - c);
        }
        if (t < 0.0) {
            return -t;
        }
    }
    return -1.0;
}

fn intersect_sphere(sphere: Sphere, ray_origin: vec3<f32>, ray_direction: vec3<f32>, t_min: f32, t_max: f32) -> Hit {
    var hit: Hit;
    hit.distance = -1.0;
    hit.material_ior = 0.0;
    
    let t = sphere_distance(sphere, ray_origin, ray_direction);
    if (t_min < t && t < t_max) {
        hit.distance = t;
        hit.position = ray_origin + ray_direction * t;
        hit.normal = normalize(hit.position - sphere.position);
        hit.material_type = sphere.material.material_type;
        hit.material_color = sphere.material.color;
        hit.material_ior = sphere.material.pad3;
    }
    return hit;
}

fn find_nearest_intersection(ray_origin: vec3<f32>, ray_direction: vec3<f32>, t_min: f32, t_max: f32) -> Hit {
    var nearest_hit: Hit;
    nearest_hit.distance = -1.0;
    
    for (var i = 0u; i < config.sphere_count; i = i + 1u) {
        let hit = intersect_sphere(spheres[i], ray_origin, ray_direction, t_min, t_max);
        if (hit.distance > 0.0) {
            if (nearest_hit.distance < 0.0 || hit.distance < nearest_hit.distance) {
                nearest_hit = hit;
            }
        }
    }
    
    if (nearest_hit.distance > 0.0) {
        if (nearest_hit.material_type != 3u) {
            if (dot(ray_direction, nearest_hit.normal) > 0.0) {
                nearest_hit.normal = nearest_hit.normal * -1.0;
            }
        }
    }
    
    return nearest_hit;
}

fn get_camera_ray(coord: vec2<u32>, size: vec2<f32>, u1: f32, u2: f32) -> Ray {
    let dx = (f32(coord.x) + u1) / (size.x / 2.0) - 1.0;
    let dy = -(f32(coord.y) + u2) / (size.y / 2.0) + 1.0;
    
    let dir = vec3<f32>(dx, dy, camera.flen);
    let world_dir = (camera.camera_to_world * vec4<f32>(dir, 0.0)).xyz;
    
    var ray: Ray;
    ray.origin = camera.position;
    ray.direction = normalize(world_dir);
    return ray;
}

fn reflectance(cosine: f32, refraction_ratio: f32) -> f32 {
    var r0 = (1.0 - refraction_ratio) / (1.0 + refraction_ratio);
    r0 = r0 * r0;
    return r0 + (1.0 - r0) * pow(1.0 - cosine, 5.0);
}

fn trace(ray_origin_in: vec3<f32>, ray_direction_in: vec3<f32>, rng_state: ptr<function, vec3<u32>>) -> vec3<f32> {
    var origin = ray_origin_in;
    var direction = ray_direction_in;
    var throughput = vec3<f32>(1.0, 1.0, 1.0);
    var accumulated_light = vec3<f32>(0.0, 0.0, 0.0);
    
    for (var depth = 0u; depth <= 10u; depth = depth + 1u) {
        let hit = find_nearest_intersection(origin, direction, 0.001, 3.40282347e+38); // f32::MAX
        
        if (hit.distance > 0.0) {
            if (hit.material_type == 0u) { // Diffuse
                let basis = tangentspace_basis(hit.normal);
                let u1 = rng_next_f32(rng_state);
                let u2 = rng_next_f32(rng_state);
                let dir = sample_hemisphere_cosine(u1, u2);
                
                origin = hit.position;
                direction = dir.x * basis[0] + dir.y * basis[1] + dir.z * hit.normal;
                throughput = throughput * hit.material_color;
            } else if (hit.material_type == 1u) { // Specular
                origin = hit.position;
                direction = direction - 2.0 * dot(direction, hit.normal) * hit.normal;
                throughput = throughput * hit.material_color;
            } else if (hit.material_type == 2u) { // Emissive
                accumulated_light = accumulated_light + throughput * hit.material_color;
                break;
            } else if (hit.material_type == 3u) { // Refractive
                let is_entering = dot(direction, hit.normal) < 0.0;
                var normal = hit.normal;
                if (!is_entering) {
                    normal = hit.normal * -1.0;
                }
                var refraction_ratio = 1.0 / hit.material_ior;
                if (!is_entering) {
                    refraction_ratio = hit.material_ior;
                }

                let cos_theta = min(dot(direction * -1.0, normal), 1.0);
                let sin_theta = sqrt(1.0 - cos_theta * cos_theta);

                let cannot_refract = refraction_ratio * sin_theta > 1.0;
                var next_direction: vec3<f32>;
                let r = rng_next_f32(rng_state);
                if (cannot_refract || reflectance(cos_theta, refraction_ratio) > r) {
                    next_direction = reflect(direction, normal);
                } else {
                    next_direction = refract(direction, normal, refraction_ratio);
                }

                origin = hit.position;
                direction = next_direction;
                throughput = throughput * hit.material_color;
            }
        } else {
            // Environment illumination
            if (config.environment.material_type == 2u) { // Emissive
                accumulated_light = accumulated_light + throughput * config.environment.color;
            }
            break;
        }
    }
    
    return accumulated_light;
}

// ----------------------------------------------------
// Compute Shader Entrypoint
// ----------------------------------------------------

@compute @workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let size = vec2<u32>(config.width, config.height);
    if (global_id.x >= size.x || global_id.y >= size.y) {
        return;
    }
    let coord = vec2<u32>(global_id.x, global_id.y);
    let index = coord.y * size.x + coord.x;
    
    // Initialize RNG state
    var rng_state = vec3<u32>(coord.x, coord.y, config.frame_count);
    
    // Sample jitter
    let u1 = rng_next_f32(&rng_state);
    let u2 = rng_next_f32(&rng_state);
    
    let ray = get_camera_ray(coord, vec2<f32>(size), u1, u2);
    let radiance = trace(ray.origin, ray.direction, &rng_state);
    
    var new_radiance = radiance;
    if (config.frame_count > 0u) {
        new_radiance = accumulation_buffer[index].rgb + radiance;
    }
    accumulation_buffer[index] = vec4<f32>(new_radiance, 1.0);
    
    let average_color = new_radiance / f32(config.frame_count + 1u);
    textureStore(output_texture, coord, vec4<f32>(average_color, 1.0));
}

// ----------------------------------------------------
// Render Shader (for Full-screen quad)
// ----------------------------------------------------

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let uv = vec2<f32>(
        f32((vertex_index << 1u) & 2u),
        f32(vertex_index & 2u)
    );
    out.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    out.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.uv);
}
