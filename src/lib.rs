use std::{cell::RefCell, iter, rc::Rc, sync::Arc};

use bytemuck::Pod;
use glam::{Mat4, Vec2, Vec3, vec2};
use glfw::{Action, Key, WindowEvent};
use log::info;
use wgpu::{Color, PrimitiveState, TextureFormat, TextureUsages};

use crate::{
    block::{Block, BlockTextureAtlas},
    chunk::Chunk,
    graphics::{
        Wgpu,
        camera::Camera,
        image::Image,
        lowlevel::{
            WgpuInstance,
            buf::{UniformBuffer, VertexLayout},
        },
        mesh::BlockVertex,
        textures::Textures,
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
    block_textures: Textures<'a>,
    blocks_bind_group: wgpu::BindGroup,
}

impl<'a> QuackCraft<'a> {
    /// Creates a new game instance.
    pub fn new(
        window: window::GlfwWindow,
        wgpu: Rc<WgpuInstance<'static>>,
    ) -> anyhow::Result<&'static mut QuackCraft<'static>> {
        let program = wgpu.load_shader(
            include_str!("../shaders/test.wgsl"),
            Some("test_shader"),
            Some("vs_main"),
            Some("fs_main"),
            wgpu::PipelineCompilationOptions::default(),
        );

        let keyboard = Rc::new(RefCell::new(Keyboard::new()));
        let camera = Rc::new(RefCell::new(CameraController::new(wgpu.clone())));
        let (camera_layout, camera_bind_group) = camera.borrow().bind_group(0);

        let closure_camera = camera.clone();
        CameraController::register_callback(closure_camera.clone(), &window);

        window.set_mouse_mode(glfw::CursorMode::Disabled);

        let mut blocks = Textures::new(wgpu.clone(), Some("block textures"), (16, 16));

        assert_eq!(
            blocks.push_invalid_texture(),
            0,
            "invalid texture not in slot zero"
        );

        let dirt_handle = blocks.add_texture(
            "dirt",
            Image::from_mem(include_bytes!("../dirt.png"))
                .expect("unable to read dirt")
                .pixel_bytes()
                .clone(),
        );

        let grass_top = blocks.add_texture(
            "grass_top",
            Image::from_mem(include_bytes!("../grass_block_top.png"))
                .expect("unable to read grass_top")
                .pixel_bytes()
                .clone(),
        );

        let grass_side = blocks.add_texture(
            "grass_side",
            Image::from_mem(include_bytes!("../grass_block_side.png"))
                .expect("unable to read grass_side")
                .pixel_bytes()
                .clone(),
        );

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
            &program,
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

        let mut world = World::empty(wgpu.clone());

        let mut chunk = Chunk::empty(wgpu.clone());
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    chunk.data[i + 4][j + 4][k + 4] = if (i + j + k) % 2 == 0 {
                        Block::Dirt
                    } else {
                        Block::Grass
                    }
                }
            }
        }

        for x in -16..16 {
            for y in -8..8 {
                for z in -16..16 {
                    world.push_chunk((x, y, z), chunk.clone());
                }
            }
        }

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
