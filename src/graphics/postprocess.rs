use glam::{Vec2, vec2};
use wgpu::{ColorTargetState, PrimitiveState, SurfaceTexture, TextureUsages};

use crate::graphics::{
    Wgpu,
    lowlevel::{
        buf::{IndexBuffer, VertexBuffer, VertexLayout},
        shader::ShaderProgram,
        texture::Texture,
    },
};

/// Module for anything past the main rendering pipeline, such as
/// copying the full screen texture to the swap chain, or
/// applying post-processing effects.
pub struct PostProcessingPass<'a> {
    #[allow(dead_code)] // If we drop this wgpu will panic on render.
    shader: ShaderProgram<'a>,
    display_texture: Texture<'a>,
    display_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    vertex_buf: VertexBuffer<Uv>,
    index_buf: IndexBuffer<u16>,
    wgpu: Wgpu<'a>,
}

const UV_VERTICES: &[Uv] = &[
    Uv(vec2(-1.0, -1.0), vec2(0.0, 1.0)),
    Uv(vec2(1.0, -1.0), vec2(1.0, 1.0)),
    Uv(vec2(-1.0, 1.0), vec2(0.0, 0.0)),
    Uv(vec2(1.0, 1.0), vec2(1.0, 0.0)),
];

const UV_INDICES: &[u16] = &[0, 1, 2, 2, 1, 3];

impl<'a> PostProcessingPass<'a> {
    pub fn new(wgpu: Wgpu<'a>) -> Self {
        let shader = wgpu.load_shader(
            include_str!("../../shaders/postprocess.wgsl"),
            Some("Post processing Shader"),
            Some("vs"),
            Some("fs"),
            Default::default(),
        );

        let output_format = wgpu.config.borrow().format;

        let render_dim = wgpu.dimensions();

        let render_texture = wgpu.texture_uninit(
            Some("Render Texture"),
            output_format,
            TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            render_dim,
            1,
        );

        let (layout, display_bind_group) =
            render_texture.layout_and_bind_group(Some("Render Texture"), 1, 0);

        let pipeline_layout =
            wgpu.pipeline_layout(Some("Post processing pipeline layout"), &[&layout]);

        let pipeline = wgpu.pipeline(
            Some("Post processing"),
            &shader,
            &pipeline_layout,
            &[Uv::LAYOUT],
            PrimitiveState::default(),
            &[Some(ColorTargetState {
                format: output_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            None,
        );

        let vertex_buf = wgpu.vertex_buffer(UV_VERTICES, Some("Post processing UV vertex buffer"));
        let index_buf = wgpu.index_buffer(UV_INDICES, Some("Post processing UV index buffer"));

        Self {
            shader,
            display_texture: render_texture,
            pipeline,
            display_bind_group,
            vertex_buf,
            index_buf,
            wgpu,
        }
    }

    /// Creates a texture view for the display texture.
    pub fn create_display_texture_view(&self) -> wgpu::TextureView {
        self.display_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder) -> SurfaceTexture {
        let (surface, view) = self
            .wgpu
            .current_view()
            .expect("unable to grab current view!");
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Post processing render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.display_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buf.buffer().slice(..));
        render_pass.set_index_buffer(self.index_buf.buffer().slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..UV_INDICES.len() as u32, 0, 0..1);
        surface
    }
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Uv(Vec2, Vec2);

unsafe impl VertexLayout for Uv {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Uv>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2, // pos
            1 => Float32x2, // tex_coord
        ],
    };
}
