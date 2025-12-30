use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

use glam::Vec3;
use glfw::{Action, Key, WindowEvent};
use log::info;
use wgpu::{Color, PrimitiveState};

use crate::{
    block::{Block, BlockTextureAtlas},
    debug::{DebugProvider, DebugRenderer},
    graphics::{
        Wgpu,
        lowlevel::{WgpuInstance, buf::VertexLayout},
        mesh::BlockVertex,
        postprocess::PostProcessingPass,
        textures::TextureCollection,
    },
    input::{camera::CameraController, keyboard::Keyboard},
    resource::Resource,
    world::World,
};

/// A read-only string type.
pub type ReadOnlyString = Arc<str>;
/// A read-only slice type.
pub type ReadOnly<T> = Arc<[T]>;
/// A position in the world, in chunk coordinates.
pub type ChunkPosition = coords::BlockPosition;
/// A position in the world, in chunk coordinates.
pub type BlockPosition = coords::BlockPosition;
/// A position in the world, in floating-point coordinates.
pub type FloatPosition = Vec3;

mod block;
mod chunk;
pub mod coords;
mod debug;
pub mod graphics;
mod input;
pub mod resource;
mod window;
mod world;

/// The main game structure.
pub struct GameState {
    keyboard: Rc<RefCell<Keyboard>>,
    world: World,
    block_textures: TextureCollection,
    blocks_bind_group: wgpu::BindGroup,
    debug_renderer: DebugRenderer,
    post_process_pass: PostProcessingPass<'static>,
    delta_time: Cell<f32>,
}

impl GameState {
    /// Creates a new game instance.
    pub fn new(window: window::GlfwWindow, wgpu: Rc<WgpuInstance>) -> anyhow::Result<GameState> {
        let solid_block_chunk_shader = wgpu.load_shader(
            include_str!("../shaders/chunk_solid.wgsl"),
            Some("Chunk Solid Block Shader"),
            Some("vs"),
            Some("fs"),
            wgpu::PipelineCompilationOptions::default(),
        );

        let mut debug_renderer = debug::DebugRenderer::new(wgpu.clone())?;
        let fps = debug_renderer.add_statistic("fps", "0");
        let frametime_stat = debug_renderer.add_statistic("frametime (ms)", "0.0");

        drop(debug_renderer);
        let keyboard = Rc::new(RefCell::new(Keyboard::new()));
        let (camera, camera_layout, camera_bind_group) =
            CameraController::create_main_camera(&wgpu, &window, &mut debug_renderer, 0);

        let mut blocks = TextureCollection::new(wgpu.clone(), Some("block textures"), (16, 16));

        assert_eq!(
            blocks.push_invalid_texture(),
            0,
            "invalid texture not in slot zero"
        );

        let (dirt_handle, _) =
            blocks.load_texture("dirt", include_minecraft_texture!("block/dirt"))?;

        let (grass_top, _) = blocks.load_texture(
            "grass_top",
            include_minecraft_texture!("block/grass_block_top"),
        )?;

        let (grass_side, _) = blocks.load_texture(
            "grass_side",
            include_minecraft_texture!("block/grass_block_side"),
        )?;

        let (oak_leaves, _) =
            blocks.load_texture("oak_leaves", include_minecraft_texture!("block/oak_leaves"))?;

        let (oak_log_top, _) = blocks.load_texture(
            "oak_log_top",
            include_minecraft_texture!("block/oak_log_top"),
        )?;

        let (oak_log_side, _) =
            blocks.load_texture("oak_log_side", include_minecraft_texture!("block/oak_log"))?;

        info!("grass_top handle: {}", grass_top);
        info!("grass_side handle: {}", grass_side);
        info!("dirt handle: {}", dirt_handle);
        info!("oak_leaves handle: {}", oak_leaves);
        info!("oak_log_top handle: {}", oak_log_top);
        info!("oak_log_side handle: {}", oak_log_side);

        let mut atlas = BlockTextureAtlas::new(0);
        atlas.set_texture_handle(Block::Dirt, dirt_handle);
        atlas.set_texture_handle(Block::Grass, grass_top);
        atlas.set_texture_handle(Block::OakLeaves, oak_leaves);
        atlas.set_texture_handle(Block::OakWood, oak_log_top);

        let texture = blocks.gpu_texture();

        let (blocks_bind_layout, blocks_bind_group) =
            texture.layout_and_bind_group(Some("block textures"), 1, 0);

        let depth_texture = wgpu.depth_texture();

        let layout = wgpu.pipeline_layout(None, &[&camera_layout, &blocks_bind_layout]);
        let pipeline = wgpu.pipeline(
            Some("main pipeline"),
            &solid_block_chunk_shader,
            &layout,
            &[BlockVertex::LAYOUT],
            PrimitiveState {
                ..Default::default()
            },
            &[Some(wgpu::ColorTargetState {
                format: wgpu.config.get().format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            Some(depth_texture.state()),
        );

        let mut world = World::test(wgpu.clone());

        world.populate_neighbors();

        let post_process_pass = PostProcessingPass::new(wgpu.clone());

        world.create_debug_providers(&mut debug_renderer);

        println!("Generating chunk mesh...");

        world
            .render_state
            .borrow_mut()
            .generate_mesh(&world, &atlas);

        println!("Chunk mesh generated.");

        Ok(GameState {
            keyboard,
            world,
            block_textures: blocks,
            blocks_bind_group,
            debug_renderer,
            post_process_pass,
            delta_time: Cell::new(0.0),
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

    fn update_camera(&mut self, _frame: u64) {
        let mut camera = self.camera.get_mut();
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
            self.debug_renderer.get_mut().toggle();
        }

        camera.flush();
    }

    pub fn render(&mut self, frame: u64) -> anyhow::Result<()> {
        let frame_start = std::time::Instant::now();
        let wgpu = self.wgpu.clone();

        let mut encoder = wgpu.create_encoder(None);
        let view = self.post_process_pass.create_display_texture_view();

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

        let mut pass = wgpu.render_pass(
            Some("World Pass"),
            &mut encoder,
            &view,
            Some(self.depth_texture.attachment()),
            wgpu::LoadOp::Clear(Self::rainbow(frame)),
        );

        pass.set_bind_group(1, &self.blocks_bind_group, &[]);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_pipeline(&self.pipelines[0]);

        self.world.render_state.borrow().render(&mut pass);

        drop(pass);

        self.debug_renderer.get_mut().render(&mut encoder, &view);

        let surface = self.post_process_pass.render(&mut encoder);

        wgpu.submit_single(encoder.finish());
        surface.present();

        let frametime = frame_start.elapsed().as_secs_f32() * 1000.0;
        self.frametime_ms.update_value(format!("{:.2}", frametime));
        let fps = 1000.0 / frametime;
        self.fps.update_value(format!("{:.2}", fps));
        Ok(())
    }
}

pub fn run_game() -> anyhow::Result<()> {
    info!("Starting quackcraft");
    let window = window::GlfwWindow::new(800, 600, "Quackcraft")?;
    let wgpu = smol::block_on(WgpuInstance::new(window.window.clone()))?;
    let qc = GameState::new(window, wgpu)?;
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

/// Includes a Minecraft resource file at the given path.
/// The path should be relative to the `res/assets/minecraft/textures` directory.
#[macro_export]
macro_rules! include_minecraft_texture {
    ($res: literal) => {
        include_bytes!(concat!("../res/assets/minecraft/textures/", $res, ".png"))
    };
}
