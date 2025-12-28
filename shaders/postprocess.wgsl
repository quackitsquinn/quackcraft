// Post-processing shader that simply samples from a fullscreen texture.


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(input.position, 0.0, 1.0);
    out.uv = input.uv;
    return out;
}

@group(0) @binding(0)
var fullscreen_texture: texture_2d_array<f32>;
@group(0) @binding(1)
var sampler_fullscreen: sampler;

@fragment
fn fs(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(fullscreen_texture, sampler_fullscreen, input.uv, 0);
}
