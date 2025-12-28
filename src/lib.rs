use std::{cell::RefCell, iter, rc::Rc, sync::Arc};

use bytemuck::Pod;
use glam::{Mat4, Vec2, Vec3, vec2};
use glfw::{Action, Key, WindowEvent};
use log::info;
use wgpu::{Color, PrimitiveState, TextureFormat, TextureUsages};

use crate::{
    block::{Block, BlockTextureAtlas},
    chunk::Chunk,
    debug::DebugStatistic,
    graphics::{
        Wgpu,
        camera::Camera,
        image::Image,
        lowlevel::{
            WgpuInstance,
            buf::{UniformBuffer, VertexLayout},
        },
        mesh::BlockVertex,
        textures::TextureCollection,
    },
    input::{
        camera::CameraController,
        keyboard::{self, Keyboard},
    },
    world::World,
};

/// A read-only string type.
pub type ReadOnlyString = Arc<str>;
/// A read-only slice type.
pub type ReadOnly<T> = Arc<[T]>;
/// A position in the world, in chunk coordinates.
pub type ChunkPosition = (i64, i64, i64);
/// A position in the world, in chunk coordinates.
pub type BlockPosition = (i64, i64, i64);
/// A position in the world, in floating-point coordinates.
pub type FloatPosition = Vec3;

mod block;
mod chunk;
mod debug;
mod graphics;
mod input;
mod window;
mod world;

/// The main game structure.
pub struct QuackCraft<'a> {
    window: window::GlfwWindow,
    wgpu: Wgpu<'a>,
    keyboard: Rc<RefCell<Keyboard>>,
    pipelines: Vec<wgpu::RenderPipeline>,
    world: World<'a>,
    depth_texture: graphics::lowlevel::depth::DepthTexture<'a>,
    camera: Rc<RefCell<CameraController<'a>>>,
    camera_bind_group: wgpu::BindGroup,
    #[allow(dead_code)] // This is used, but through weird chains of Rc borrows.
    block_textures: TextureCollection<'a>,
    blocks_bind_group: wgpu::BindGroup,
    debug_renderer: debug::DebugRenderer<'a>,
}

impl<'a> QuackCraft<'a> {
    /// Creates a new game instance.
    pub fn new(
        window: window::GlfwWindow,
        wgpu: Rc<WgpuInstance<'static>>,
    ) -> anyhow::Result<&'static mut QuackCraft<'static>> {
        let block_shader = wgpu.load_shader(
            include_str!("../shaders/block.wgsl"),
            Some("block_shader"),
            Some("vs_main"),
            Some("fs_main"),
            wgpu::PipelineCompilationOptions::default(),
        );

        let keyboard = Rc::new(RefCell::new(Keyboard::new()));
        let (camera, camera_layout, camera_bind_group) =
            CameraController::create_main_camera(&wgpu, &window, 0);

        let mut blocks = TextureCollection::new(wgpu.clone(), Some("block textures"), (16, 16));

        assert_eq!(
            blocks.push_invalid_texture(),
            0,
            "invalid texture not in slot zero"
        );

        let (dirt_handle, _) = blocks.load_texture("dirt", include_bytes!("../dirt.png"))?;

        let (grass_top, _) =
            blocks.load_texture("grass_top", include_bytes!("../grass_block_top.png"))?;

        let (grass_side, _) =
            blocks.load_texture("grass_side", include_bytes!("../grass_block_side.png"))?;

        info!("grass_top handle: {}", grass_top);
        info!("grass_side handle: {}", grass_side);
        info!("dirt handle: {}", dirt_handle);

        let mut atlas = BlockTextureAtlas::new(0);
        atlas.set_texture_handle(Block::Dirt, dirt_handle);
        atlas.set_texture_handle(Block::Grass, grass_top);

        let texture = blocks.gpu_texture();

        let (blocks_bind_layout, blocks_bind_group) =
            texture.layout_and_bind_group(Some("block textures"), 1, 0);

        let depth_texture = wgpu.depth_texture();

        let layout = wgpu.pipeline_layout(None, &[&camera_layout, &blocks_bind_layout]);
        let pipeline = wgpu.pipeline(
            Some("main pipeline"),
            &block_shader,
            &layout,
            &[BlockVertex::LAYOUT],
            PrimitiveState {
                ..Default::default()
            },
            &[Some(wgpu::ColorTargetState {
                format: wgpu.config.borrow().format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            Some(depth_texture.state()),
        );

        let world = World::test(wgpu.clone());

        let mut debug_renderer = debug::DebugRenderer::new(wgpu.clone())?;

        println!("Generating chunk mesh...");

        world
            .render_state
            .borrow_mut()
            .generate_mesh(&world, &atlas);

        println!("Chunk mesh generated.");

        Ok(Box::leak(Box::new(QuackCraft {
            window,
            wgpu: wgpu.clone(),
            pipelines: vec![pipeline],
            keyboard,
            camera,
            camera_bind_group,
            depth_texture,
            world,
            block_textures: blocks,
            debug_renderer,
            blocks_bind_group,
        })))
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
        let keyboard = self.keyboard.borrow();
        let speed = 0.2;
        let front = camera.front();
        if keyboard.is_key_held(Key::W) {
            let front = camera.front();
            camera.update_position(|c| c + front * speed);
        }
        if keyboard.is_key_held(Key::S) {
            let front = camera.front();
            camera.update_position(|c| c - front * speed);
        }
        if keyboard.is_key_held(Key::A) {
            let right = front.cross(Vec3::Y).normalize();
            camera.update_position(|c| c - right * speed);
        }
        if keyboard.is_key_held(Key::D) {
            let right = front.cross(Vec3::Y).normalize();
            camera.update_position(|c| c + right * speed);
        }

        if keyboard.is_key_pressed(Key::F3) {
            self.debug_renderer.toggle();
        }

        camera.flush();
    }

    pub fn render(&mut self, frame: u64) -> anyhow::Result<()> {
        let wgpu = self.wgpu.clone();

        let mut encoder = wgpu.create_encoder(None);
        let (surface, view) = wgpu.current_view()?;

        if self.keyboard.borrow().is_key_pressed(Key::Escape) {
            match self.window.get_mouse_mode() {
                glfw::CursorMode::Disabled => {
                    // unpause
                    self.window.set_mouse_mode(glfw::CursorMode::Normal);
                    self.window.mouse_pos_proxy.suspend();
                }
                glfw::CursorMode::Normal => {
                    // pause
                    self.window.set_mouse_mode(glfw::CursorMode::Disabled);
                    self.window.mouse_pos_proxy.unsuspend();
                }
                _ => {}
            }
        }

        self.update_camera(frame);

        let mut pass = wgpu.start_main_pass(
            Self::rainbow(frame),
            &mut encoder,
            &view,
            Some(self.depth_texture.attachment()),
        );

        pass.set_bind_group(1, &self.blocks_bind_group, &[]);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_pipeline(&self.pipelines[0]);

        self.world.render_state.borrow().render(&mut pass);

        drop(pass);

        self.debug_renderer.render(&mut encoder, &view);

        wgpu.submit_single(encoder.finish());
        surface.present();

        Ok(())
    }
}

pub fn run_game() -> anyhow::Result<()> {
    info!("Starting quackcraft");
    let window = window::GlfwWindow::new(800, 600, "Quackcraft")?;
    let wgpu = smol::block_on(WgpuInstance::new(window.window.clone()))?;
    let qc = QuackCraft::new(window, wgpu)?;
    let mut frame: u64 = 0;
    while !qc.window.should_close() {
        qc.window.poll_events();
        let mut keyboard = qc.keyboard.borrow_mut();
        keyboard.update_keys();
        while let Some((_, event)) = qc.window.event_receiver.receive() {
            match event {
                WindowEvent::Close => break,
                WindowEvent::Size(x, y) => {
                    qc.wgpu.resize((x, y));
                }
                WindowEvent::Key(key, _, Action::Press, _) => {
                    keyboard.press_key(key);
                }

                WindowEvent::Key(key, _, Action::Release, _) => {
                    keyboard.release_key(key);
                }
                _ => {}
            }
        }
        drop(keyboard);
        qc.render(frame)?;
        frame += 1;
    }
    Ok(())
}
