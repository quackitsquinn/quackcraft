/// Solid block chunk shader.

/// Draw data passed from vertex to fragment shader.
struct DrawData {
    /// Clip space position.
    @builtin(position) clip_position: vec4<f32>,
    /// Texture coordinates.
    @location(0) tex_coord: vec2<f32>,
    /// Texture ID for the block type. Specifically, the index into `block_textures`.
    @location(1) texture_id: u32,
}

/// Vertex shader
/// Shader inputs and outputs
struct ChunkData {
    /// Position of the vertex in world space.
    @location(0) position: vec3<f32>,
    /// Texture coordinates.
    @location(1) tex_coord: vec2<f32>,
    /// Texture ID for the block type. Specifically, the index into `block_textures`.
    @location(2) texture_id: u32,
}

@group(0) @binding(0) // Camera uniform buffer
var<uniform> camera: mat4x4<f32>;

@vertex
fn vs(
    chunk: ChunkData,
) -> DrawData {
    var draw: DrawData;
    draw.tex_coord = chunk.tex_coord;
    draw.texture_id = chunk.texture_id;
    draw.clip_position = camera * vec4<f32>(chunk.position, 1.0);
    return draw;
}


// /// Fragment shader
// @group(1) @binding(0) // Block texture array
// var block_textures: texture_2d_array<f32>;
// @group(1) @binding(1) // Block texture sampler
// var sampler_block: sampler;

@fragment
fn fs(in: DrawData) -> @location(0) vec4<f32> {
    return vec4<f32>(in.tex_coord, f32(in.texture_id) / 16.0, 1.0);
}


 