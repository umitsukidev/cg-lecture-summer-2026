struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

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

fn quantize_n_levels(value: f32, n: f32) -> f32 {
    if (n < 2.0) {
        return 0.0;
    }
    let step = 1.0 / (n - 1.0);
    let index = round(value / step);
    return index * step;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 2D Gaussian blur approximation
    // Kernel size = 17x17 (radius = 8), stepping by 2.0 to cover a 33x33 area.
    let radius: i32 = 8;
    let sigma: f32 = 6.0;
    let two_sigma_sq = 2.0 * sigma * sigma;
    let size = 1024.0;
    
    var sum: f32 = 0.0;
    var total_weight: f32 = 0.0;
    
    for (var x = -radius; x <= radius; x = x + 1) {
        for (var y = -radius; y <= radius; y = y + 1) {
            let offset = vec2<f32>(f32(x) * 2.0, f32(y) * 2.0) / size;
            let weight = exp(-f32(x * x + y * y) / two_sigma_sq);
            
            // Texture has black circles on white background, so we want the circle intensity (1.0 - color.r)
            let tex_color = textureSample(t_diffuse, s_diffuse, in.uv + offset);
            let val = 1.0 - tex_color.r;
            
            sum = sum + val * weight;
            total_weight = total_weight + weight;
        }
    }
    
    let blurred = sum / total_weight;
    
    // Metaball thresholding:
    // Any pixel with blurred value > threshold forms part of the metaball.
    let threshold = 0.35;
    var value: f32 = 0.0;
    
    if (blurred > threshold) {
        // Map the range [threshold, 1.0] to [0.0, 1.0]
        let intensity = (blurred - threshold) / (1.0 - threshold);
        // Quantize to 4 levels
        if (quantize_n_levels(intensity, 8.0) > 0.0) {
            value = 255.0;
        } else {
            value = 0.0;
        }
    }
    
    // Output black metaballs on a white background
    let out_color = 1.0 - value;
    return vec4<f32>(out_color, out_color, out_color, 1.0);
}