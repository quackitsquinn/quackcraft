use std::{cell::RefCell, iter, rc::Rc, sync::Arc};

use bytemuck::Pod;
use glam::{Mat4, Vec3};
use glfw::WindowEvent;
use log::info;
use wgpu::{Color, TextureFormat, TextureUsages};

use crate::graphics::{
    camera::Camera,
    lowlevel::{
        WgpuInstance,
        buf::{IndexBuffer, UniformBuffer, VertexBuffer, VertexLayout},
    },
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
    wgpu: Rc<graphics::lowlevel::WgpuInstance<'a>>,
    pipelines: Vec<wgpu::RenderPipeline>,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    depth_texture: graphics::lowlevel::depth::DepthTexture<'a>,
    dirt_image: graphics::image::Image,
    dirt_texture: graphics::lowlevel::texture::Texture<'a>,
    dirt_bind_group: (wgpu::BindGroupLayout, wgpu::BindGroup),
    camera: RefCell<Camera>,
    transform_uniform: UniformBuffer<'a, Mat4>,
    camera_bind_group: wgpu::BindGroup,
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x2, // tex_coords
    ];

    pub const fn new(position: [f32; 3], tex_coords: [f32; 2]) -> Self {
        Self {
            position,
            tex_coords,
        }
    }
}
const CUBE_VERTICES: &[Vertex] = &[
    // Front (+Z)
    Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0]),
    Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0]),
    Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0]),
    Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0]),
    // Back (-Z)
    Vertex::new([1.0, -1.0, -1.0], [0.0, 0.0]),
    Vertex::new([-1.0, -1.0, -1.0], [1.0, 0.0]),
    Vertex::new([-1.0, 1.0, -1.0], [1.0, 1.0]),
    Vertex::new([1.0, 1.0, -1.0], [0.0, 1.0]),
    // Left (-X)
    Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0]),
    Vertex::new([-1.0, -1.0, 1.0], [1.0, 0.0]),
    Vertex::new([-1.0, 1.0, 1.0], [1.0, 1.0]),
    Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0]),
    // Right (+X)
    Vertex::new([1.0, -1.0, 1.0], [0.0, 0.0]),
    Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0]),
    Vertex::new([1.0, 1.0, -1.0], [1.0, 1.0]),
    Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0]),
    // Top (+Y)
    Vertex::new([-1.0, 1.0, 1.0], [0.0, 0.0]),
    Vertex::new([1.0, 1.0, 1.0], [1.0, 0.0]),
    Vertex::new([1.0, 1.0, -1.0], [1.0, 1.0]),
    Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0]),
    // Bottom (-Y)
    Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0]),
    Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0]),
    Vertex::new([1.0, -1.0, 1.0], [1.0, 1.0]),
    Vertex::new([-1.0, -1.0, 1.0], [0.0, 1.0]),
];

pub const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, // Front
    4, 5, 6, 6, 7, 4, // Back
    8, 9, 10, 10, 11, 8, // Left
    12, 13, 14, 14, 15, 12, // Right
    16, 17, 18, 18, 19, 16, // Top
    20, 21, 22, 22, 23, 20, // Bottom
];

unsafe impl VertexLayout for Vertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: Self::ATTRIBUTES,
    };
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

        let vertex_buf = wgpu.vertex_buffer(CUBE_VERTICES, Some("vertex buffer"));

        let ibuf = wgpu.index_buffer(CUBE_INDICES, Some("index buffer"));

        let dirt_image = graphics::image::Image::from_mem(include_bytes!("../dirt.png"))?;

        let dirt_texture = wgpu.texture(
            Some("dirt texture"),
            TextureFormat::Rgba8UnormSrgb,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            &dirt_image,
        );

        let dirt_bind_group = dirt_texture.layout_and_bind_group(Some("dirt"), 0, 1);

        let layout = wgpu.pipeline_layout(None, &[&dirt_bind_group.0, &camera_layout]);

        let depth_texture = wgpu.depth_texture();

        let pipeline = wgpu.pipeline(
            Some("main pipeline"),
            &program,
            &layout,
            &[vertex_buf.layout()],
            wgpu::PrimitiveState::default(),
            &[Some(wgpu::ColorTargetState {
                format: wgpu.config.borrow().format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            Some(depth_texture.state()),
        );

        Ok(QuackCraft {
            window,
            dirt_texture,
            wgpu: wgpu.clone(),
            pipelines: vec![pipeline],
            vertex_buffer: vertex_buf,
            index_buffer: ibuf,
            dirt_image,
            dirt_bind_group,
            camera: RefCell::new(camera),
            transform_uniform: camera_buf,
            camera_bind_group,
            depth_texture,
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
        let angle = (frame as f32) * 0.02;
        let radius = 5.0;
        let mut camera = self.camera.borrow_mut();
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        camera.pos(Vec3::new(x, 3.0, z));
        camera.look_at(Vec3::new(0.0, 0.0, 0.0));
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

        pass.set_bind_group(0, &self.dirt_bind_group.1, &[]);
        pass.set_bind_group(1, &self.camera_bind_group, &[]);
        pass.set_pipeline(&self.pipelines[0]);
        self.vertex_buffer.set_on(&mut pass, 0, ..);
        self.index_buffer.set_on(&mut pass, ..);
        pass.draw_indexed(0..CUBE_INDICES.len() as u32, 0, 0..1);

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
