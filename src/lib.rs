use std::{cell::RefCell, iter, rc::Rc, sync::Arc};

use bytemuck::Pod;
use glam::{Mat4, Vec3};
use glfw::WindowEvent;
use log::info;
use wgpu::{Color, PrimitiveState, TextureFormat, TextureUsages};

use crate::{
    block::Block,
    chunk::Chunk,
    graphics::{
        camera::Camera,
        lowlevel::{
            WgpuInstance,
            buf::{UniformBuffer, VertexLayout},
        },
        mesh::BlockVertex,
    },
};

/// A read-only string type.
pub type ReadOnlyString = Arc<str>;
/// A read-only slice type.
pub type ReadOnly<T> = Arc<[T]>;
/// A position in the world, in chunk coordinates.
pub type BlockPosition = (i64, i64, i64);
/// A position in the world, in floating-point coordinates.
pub type FloatPosition = Vec3;

mod block;
mod chunk;
mod graphics;
mod window;

/// The main game structure.
pub struct QuackCraft<'a> {
    window: window::GlfwWindow,
    wgpu: Rc<graphics::lowlevel::WgpuInstance<'a>>,
    pipelines: Vec<wgpu::RenderPipeline>,
    chunk: Chunk<'a>,
    depth_texture: graphics::lowlevel::depth::DepthTexture<'a>,
    camera: RefCell<Camera>,
    transform_uniform: UniformBuffer<'a, Mat4>,
    camera_bind_group: wgpu::BindGroup,
}

impl<'a> QuackCraft<'a> {
    /// Creates a new game instance.
    pub fn new(window: window::GlfwWindow, wgpu: Rc<WgpuInstance<'a>>) -> anyhow::Result<Self> {
        let program = wgpu.load_shader(
            include_str!("../shaders/test.wgsl"),
            Some("test_shader"),
            Some("vs_main"),
            Some("fs_main"),
            wgpu::PipelineCompilationOptions::default(),
        );

        let mut camera = Camera::new(
            wgpu.config.borrow().width as f32 / wgpu.config.borrow().height as f32,
            0.1,
            100.0,
        );

        camera.pos(Vec3::new(3.0, 3.0, 3.0));
        camera.look_at(Vec3::new(0.0, 0.0, 0.0));

        let camera_layout = wgpu.bind_group_layout(
            Some("camera bind group layout"),
            &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        );

        let camera_buf =
            wgpu.uniform_buffer(&camera.projection_view_matrix(), Some("camera buffer"));

        let camera_bind_group = wgpu.bind_group(
            Some("camera bind group"),
            &camera_layout,
            &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    camera_buf.buffer().as_entire_buffer_binding(),
                ),
            }],
        );

        let layout = wgpu.pipeline_layout(None, &[&camera_layout]);

        let depth_texture = wgpu.depth_texture();

        let pipeline = wgpu.pipeline(
            Some("main pipeline"),
            &program,
            &layout,
            &[BlockVertex::LAYOUT],
            PrimitiveState {
                polygon_mode: wgpu::PolygonMode::Line,
                ..Default::default()
            },
            &[Some(wgpu::ColorTargetState {
                format: wgpu.config.borrow().format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            Some(depth_texture.state()),
        );

        let mut chunk = Chunk::empty((0, 0, 0), wgpu.clone());

        for i in 0..8 {
            for j in 0..8 {
                for k in 0..8 {
                    chunk.data[i + 4][j + 4][k + 4] = if k % 2 == 0 {
                        Block::Dirt
                    } else {
                        Block::Stone
                    }
                }
            }
        }

        println!("Generating chunk mesh...");

        chunk.render_state.borrow_mut().generate_mesh(&chunk);

        Ok(QuackCraft {
            window,
            wgpu: wgpu.clone(),
            pipelines: vec![pipeline],
            camera: RefCell::new(camera),
            transform_uniform: camera_buf,
            camera_bind_group,
            depth_texture,
            chunk,
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

    fn update_camera(&mut self, frame: u64) {
        let mut camera = self.camera.borrow_mut();
        camera.pos(Vec3::new(12.0, 12.0, 0.0));
        camera.look_at(Vec3::new(8.0, 8.0, 8.0));
        let matrix = camera.projection_view_matrix();
        self.transform_uniform.write(&matrix);
    }

    pub fn render(&mut self, frame: u64) -> anyhow::Result<()> {
        let wgpu = self.wgpu.clone();

        let mut encoder = wgpu.create_encoder(None);
        let (surface, view) = wgpu.current_view()?;

        self.update_camera(frame);

        let mut pass = wgpu.start_main_pass(
            Self::rainbow(frame),
            &mut encoder,
            &view,
            Some(self.depth_texture.attachment()),
        );

        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_pipeline(&self.pipelines[0]);
        let (vertex_buffer, index_buffer) = self.chunk.render_state.borrow().buffers();
        vertex_buffer.set_on(&mut pass, 0, ..);
        index_buffer.set_on(&mut pass, ..);
        pass.draw_indexed(0..index_buffer.count() as u32, 0, 0..1);

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
