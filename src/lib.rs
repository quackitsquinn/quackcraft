use std::{cell::RefCell, iter, rc::Rc, sync::Arc};

use bytemuck::Pod;
use glfw::WindowEvent;
use log::info;
use wgpu::{Color, TextureFormat, TextureUsages};

use crate::graphics::{
    WgpuInstance,
    buf::{BufferLayout, Index16, ShaderType},
};

/// A read-only string type.
pub type ReadOnlyString = Arc<str>;
/// A read-only slice type.
pub type ReadOnly<T> = Arc<[T]>;

mod graphics;
mod window;

/// The main game structure.
pub struct QuackCraft<'a> {
    window: window::GlfwWindow,
    wgpu: Rc<graphics::WgpuInstance<'a>>,
    pipelines: Vec<wgpu::RenderPipeline>,
    vertex_buffer: graphics::buf::WgpuBuffer<Vertex>,
    index_buffer: graphics::buf::WgpuBuffer<Index16>,
    dirt_image: graphics::image::Image,
    dirt_texture: graphics::texture::Texture<'a>,
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // color
    ];

    pub const fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self { position, color }
    }

    // TODO: color correction
    pub const fn new_rgb(position: [f32; 3], rgb: [f32; 3]) -> Self {
        Self {
            position,
            color: [rgb[0], rgb[1], rgb[2]],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex::new_rgb([0.5, 0.5, 0.0], [1.0, 1.0, 0.0]),
    Vertex::new_rgb([-0.5, 0.5, 0.0], [0.0, 0.0, 1.0]),
    Vertex::new_rgb([-0.5, -0.5, 0.0], [1.0, 0.0, 0.0]),
    Vertex::new_rgb([0.5, -0.5, 0.0], [0.0, 1.0, 0.0]),
];

const INDICES: &[u16] = &[
    0, 1, 2, // first triangle
    0, 2, 3, // second triangle
];

unsafe impl ShaderType for Vertex {
    fn layout() -> BufferLayout {
        BufferLayout::Vertex(wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBUTES,
        })
    }
}

impl<'a> QuackCraft<'a> {
    /// Creates a new game instance.
    pub fn new(window: window::GlfwWindow, wgpu: Rc<WgpuInstance<'a>>) -> anyhow::Result<Self> {
        let wgpu_ret = wgpu.clone();

        let program = wgpu.load_shader(
            include_str!("../shaders/test.wgsl"),
            Some("test_shader"),
            Some("vs_main"),
            Some("fs_main"),
            wgpu::PipelineCompilationOptions::default(),
        );

        let vbuf = wgpu.create_buffer(wgpu::BufferUsages::VERTEX, VERTICES, Some("vertex buffer"));

        let ibuf = wgpu.create_buffer(
            wgpu::BufferUsages::INDEX,
            bytemuck::cast_slice::<_, Index16>(INDICES),
            Some("index buffer"),
        );

        let layout = wgpu.pipeline_layout(None, &[]);

        let pipeline = wgpu.pipeline(
            Some("main pipeline"),
            &program,
            &layout,
            &[vbuf.layout().as_vertex().expect("infallible")],
            wgpu::PrimitiveState::default(),
            &[Some(wgpu::ColorTargetState {
                format: wgpu.config.borrow().format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        );

        let dirt_image = graphics::image::Image::from_mem(include_bytes!("../dirt.png"))?;

        let dirt_texture = wgpu.texture(
            Some("dirt texture"),
            TextureFormat::Rgba8Uint,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            &dirt_image,
        );

        Ok(QuackCraft {
            window,
            dirt_texture,
            wgpu: wgpu.clone(),
            pipelines: vec![pipeline],
            vertex_buffer: vbuf,
            index_buffer: ibuf,
            dirt_image,
        })
    }

    fn rainbow(frame: u64) -> Color {
        let t = (frame as f64) * 0.02;
        Color {
            r: 0.5 + 0.5 * (t).sin(),
            g: 0.5 + 0.5 * (t + 2.0).sin(),
            b: 0.5 + 0.5 * (t + 4.0).sin(),
            a: 1.0,
        }
    }

    pub fn render(&mut self, frame: u64) -> anyhow::Result<()> {
        let wgpu = self.wgpu.clone();

        let mut encoder = wgpu.create_encoder(None);
        let (surface, view) = wgpu.current_view()?;

        let mut pass = wgpu.start_main_pass(Self::rainbow(frame), &mut encoder, &view);

        pass.set_pipeline(&self.pipelines[0]);
        pass.set_vertex_buffer(0, self.vertex_buffer.buffer().slice(..));
        pass.set_index_buffer(
            self.index_buffer.buffer().slice(..),
            wgpu::IndexFormat::Uint16,
        );
        pass.draw_indexed(0..6, 0, 0..1);

        drop(pass);

        wgpu.submit_single(encoder.finish());
        surface.present();

        Ok(())
    }
}

pub fn run_game() -> anyhow::Result<()> {
    info!("Starting quackcraft");
    let window = window::GlfwWindow::new(800, 600, "Quackcraft")?;
    let wgpu = smol::block_on(WgpuInstance::new(window.window.clone()))?;
    let mut qc = QuackCraft::new(window, wgpu)?;
    let mut frame: u64 = 0;
    while !qc.window.should_close() {
        qc.window.poll_events();
        if let Some((_, event)) = qc.window.event_receiver.receive() {
            info!("Event: {:?}", event);
            match event {
                WindowEvent::Close => break,
                WindowEvent::Size(x, y) => {
                    qc.wgpu.resize((x, y));
                }

                _ => {}
            }
        }
        qc.render(frame)?;
        frame += 1;
    }
    Ok(())
}
