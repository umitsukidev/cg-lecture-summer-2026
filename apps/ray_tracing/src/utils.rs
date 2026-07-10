use crate::{
    camera::Camera, hit::Hit, material::Material, nannou_utils::Point3Ext, ray::Ray, sphere::Sphere,
};
use nannou::{image::Rgba, prelude::*};

pub fn find_nearest_intersection<'a>(
    spheres: &[Sphere<'a>],
    ray: Ray,
    t_min: f32,
    t_max: f32,
) -> Option<Hit<'a>> {
    let mut hit = spheres
        .iter()
        .filter_map(|sphere| sphere.intersect(ray, t_min, t_max))
        .min_by(|a, b| a.distance.total_cmp(&b.distance));

    if let Some(hit) = &mut hit {
        match hit.material {
            Material::Refractive { .. } => {}
            _ => {
                if ray.direction.dot(hit.normal) > 0.0 {
                    hit.normal *= -1.0;
                }
            }
        }
    }
    hit
}

pub fn render(
    window_rect: Rect,
    x: u32,
    y: u32,
    camera: &Camera,
    spheres: &[Sphere<'_>],
    environment: &Material,
    count: u64,
    pixel: &mut Vec3,
) -> Rgba<u8> {
    let view = camera.ray(window_rect, UVec2::new(x, y), random_f32(), random_f32());
    *pixel += trace(environment, spheres, view, 0);
    let final_color = *pixel / (count as f32 + 1.0);
    final_color.to_color()
}

pub fn trace(environment: &Material, spheres: &[Sphere<'_>], ray: Ray, depth: u32) -> Vec3 {
    if 10 < depth {
        return vec3(0.0, 0.0, 0.0);
    }

    let mut ray = ray;
    let hit = find_nearest_intersection(spheres, ray, 0.001, f32::MAX);
    let mut result = vec3(0.0, 0.0, 0.0);

    if let Some(hit) = hit {
        match hit.material {
            Material::Diffuse { reflection } => {
                let (t, b) = tangentspace_basis(hit.normal);
                let dir = sample_hemisphere_cosine(random_f32(), random_f32());
                ray.origin = hit.position;
                ray.direction = dir.x * t + dir.y * b + dir.z * hit.normal;
                result += trace(environment, spheres, ray, depth + 1) * *reflection;
            }
            Material::Specular { reflection } => {
                let direction = ray.direction;
                let normal = hit.normal;
                ray.origin = hit.position;
                ray.direction = direction - 2.0 * (direction.dot(normal)) * normal;
                result += trace(environment, spheres, ray, depth + 1) * *reflection;
            }
            Material::Emissive { emission } => {
                result += *emission;
            }
            Material::Refractive { reflection, ior } => {
                let is_entering = ray.direction.dot(hit.normal) < 0.0;
                let normal = if is_entering { hit.normal } else { -hit.normal };
                let refraction_ratio = if is_entering { 1.0 / ior } else { *ior };

                let cos_theta = (-ray.direction.dot(normal)).min(1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

                let cannot_refract = refraction_ratio * sin_theta > 1.0;
                let direction =
                    if cannot_refract || reflectance(cos_theta, refraction_ratio) > random_f32() {
                        ray.direction - 2.0 * ray.direction.dot(normal) * normal
                    } else {
                        refract(ray.direction, normal, refraction_ratio)
                    };

                ray.origin = hit.position;
                ray.direction = direction;
                result += trace(environment, spheres, ray, depth + 1) * *reflection;
            }
        }
    } else {
        match environment {
            Material::Emissive { emission } => return *emission,
            _ => return vec3(0.0, 0.0, 0.0),
        }
    }
    result
}

pub fn tangentspace_basis(n: Vec3) -> (Vec3, Vec3) {
    let sg = if n.z < 0.0 { -1.0 } else { 1.0 };
    let a_factor = -1.0 / (sg + n.z);
    let b_factor = n.x * n.y * a_factor;
    let t = vec3(1.0 + sg * n.x * n.x * a_factor, sg * b_factor, -sg * n.x);
    let b = vec3(b_factor, sg + n.y * n.y * a_factor, -n.y);
    (t, b)
}

pub fn sample_hemisphere_cosine(u1: f32, u2: f32) -> Vec3 {
    let r = u1.sqrt();
    let theta = u2 * 2.0 * PI;
    vec3(r * theta.cos(), r * theta.sin(), (1.0 - u1).sqrt())
}

fn reflectance(cosine: f32, refraction_ratio: f32) -> f32 {
    let mut r0 = (1.0 - refraction_ratio) / (1.0 + refraction_ratio);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}

fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = (-uv.dot(n)).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
    r_out_perp + r_out_parallel
}
