struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) block_type: u32,
}


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color:  vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) block_type: u32,
}

// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var block_textures: texture_2d_array<f32>;
@group(1) @binding(1)
var sampler_block: sampler;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = model.tex_coord;
    out.block_type = model.block_type;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

 

 
// Fragment shader


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(
        block_textures,
        sampler_block,
        vec2<f32>(1.0) - in.tex_coord,
        in.block_type,
    );
}


 