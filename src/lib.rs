use std::{
    any::{Any, TypeId},
    marker, mem,
};

use engine::{
    component::ComponentStore,
    graphics::{
        lowlevel::{WgpuRenderer, depth::DepthTexture},
        pipeline::{controller::RenderController, pipelines::clear::ClearPipeline},
    },
    input::{camera::CameraController, keyboard::Keyboard},
    window,
};
use glam::Vec3;
use glfw::{Action, WindowEvent};
use log::info;

use crate::{
    render::{RenderPipelines, pipelines::solid::SolidGeometryPipeline},
    world::ActiveWorld,
};

pub mod coords;
pub mod mesh;
pub mod render;
pub mod world;

/// A position in the world, in chunk coordinates.
pub type BlockPosition = coords::BlockPosition;
/// A position in the world, in chunk coordinates.
pub type ChunkPosition = coords::BlockPosition;

pub const FACE_TABLE: [[([f32; 3], [f32; 2]); 4]; 6] = [
    // +X
    [
        ([1.0, 0.0, 1.0], [0.0, 0.0]),
        ([1.0, 1.0, 1.0], [0.0, 1.0]),
        ([1.0, 1.0, 0.0], [1.0, 1.0]),
        ([1.0, 0.0, 0.0], [1.0, 0.0]),
    ],
    // -X
    [
        ([0.0, 0.0, 1.0], [0.0, 0.0]),
        ([0.0, 1.0, 1.0], [0.0, 1.0]),
        ([0.0, 1.0, 0.0], [1.0, 1.0]),
        ([0.0, 0.0, 0.0], [1.0, 0.0]),
    ],
    // +Y
    [
        ([1.0, 1.0, 0.0], [1.0, 0.0]),
        ([0.0, 1.0, 0.0], [0.0, 0.0]),
        ([0.0, 1.0, 1.0], [0.0, 1.0]),
        ([1.0, 1.0, 1.0], [1.0, 1.0]),
    ],
    // -Y
    [
        ([0.0, 0.0, 1.0], [0.0, 1.0]),
        ([1.0, 0.0, 1.0], [1.0, 1.0]),
        ([1.0, 0.0, 0.0], [1.0, 0.0]),
        ([0.0, 0.0, 0.0], [0.0, 0.0]),
    ],
    // +Z
    [
        ([0.0, 1.0, 1.0], [0.0, 1.0]),
        ([0.0, 0.0, 1.0], [0.0, 0.0]),
        ([1.0, 0.0, 1.0], [1.0, 0.0]),
        ([1.0, 1.0, 1.0], [1.0, 1.0]),
    ],
    // -Z
    [
        ([1.0, 0.0, 0.0], [1.0, 0.0]),
        ([1.0, 1.0, 0.0], [1.0, 1.0]),
        ([0.0, 1.0, 0.0], [0.0, 1.0]),
        ([0.0, 0.0, 0.0], [0.0, 0.0]),
    ],
];

pub const FACE_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

pub struct Game {
    component_db: ComponentStore,
}

impl Game {
    pub fn new() -> anyhow::Result<Self> {
        let mut state = ComponentStore::new();
        state.insert(Keyboard::new());
        let window = window::GlfwWindow::new(800, 600, "Minecraft Clone")
            .expect("Failed to create GLFW window");
        smol::block_on(WgpuRenderer::attach_to(&mut state, &window))?;
        window.set_mouse_mode(glfw::CursorMode::Disabled);
        state.insert(window);

        let camera = CameraController::new(&state);
        let camera_handle = state.insert(camera);

        let world = world::World::test(&state.handle());
        let active_world = ActiveWorld::with_world(world);
        state.insert(active_world);

        let renderer: RenderController<RenderPipelines> = RenderController::new(&state);
        state.insert(renderer);

        let depth_texture = DepthTexture::new(&state);
        state.insert(depth_texture);

        state.finish_initialization();

        let mut renderer = state.get_mut::<RenderController<RenderPipelines>>();
        renderer.add_pipeline(
            RenderPipelines::Clear,
            ClearPipeline::new(1.0, 0.0, 0.5, 1.0),
        );

        let solid_pipeline = SolidGeometryPipeline::new(&state);
        renderer.add_pipeline(RenderPipelines::SolidGeometry, solid_pipeline);

        renderer.set_render_order(vec![RenderPipelines::Clear, RenderPipelines::SolidGeometry]);

        let mut camera = state.get_mut::<CameraController>();
        camera.pos = glam::Vec3::new(30.0, 32.0, 30.0);
        camera.look_at(Vec3::ZERO);

        drop_all!(renderer, camera);

        CameraController::register_callback(camera_handle, &state.get::<window::GlfwWindow>());

        Ok(Self {
            component_db: state,
        })
    }

    /// Updates the game state.
    ///
    /// `delta_time` is the time elapsed since the last update, in seconds.
    ///
    /// Returns `Ok(())` if the update was successful, or `Err(None)` if the game should exit,
    /// or `Err(Some(error))` if an error occurred.
    pub fn update(&mut self, delta_time: f64) -> Option<()> {
        let _ = delta_time;

        let mut window = self.component_db.get_mut::<window::GlfwWindow>();
        if window.should_close() {
            return None;
        }

        let mut keyboard = self.component_db.get_mut::<Keyboard>();
        keyboard.update_keys();
        let wgpu = self.component_db.get::<WgpuRenderer>();

        window.poll_events();

        while let Some((_, event)) = window.event_receiver.receive() {
            match event {
                WindowEvent::Close => {
                    return None;
                }
                WindowEvent::Size(x, y) => {
                    wgpu.resize((x, y));
                }
                WindowEvent::Key(key, _, Action::Press, _) => {
                    info!("Key pressed: {:?}", key);
                    keyboard.press_key(key);
                }

                WindowEvent::Key(key, _, Action::Release, _) => {
                    info!("Key released: {:?}", key);
                    keyboard.release_key(key);
                }
                _ => {}
            }
        }

        // Update the camera
        let mut camera = self.component_db.get_mut::<CameraController>();
        camera.update_camera(&keyboard, delta_time);

        drop_all!(window, keyboard, camera);

        let mut renderer = self
            .component_db
            .get_mut::<RenderController<RenderPipelines>>();

        renderer.update_pipelines();
        Some(())
    }

    /// Renders a frame. Returns the time taken to render the frame, in milliseconds.
    pub fn render(&mut self) -> anyhow::Result<()> {
        let renderer = self
            .component_db
            .get_mut::<RenderController<RenderPipelines>>();
        let wgpu = self.component_db.get::<WgpuRenderer>();
        let mut encoder = wgpu.create_encoder(Some("Main Render Encoder"));
        let (view, _texture) = renderer.render_pipelines(&mut encoder)?;
        wgpu.submit_single(encoder.finish());
        view.present();
        Ok(())
    }
}

pub fn run_game() -> anyhow::Result<()> {
    let game = Game::new()?;

    println!("Game initialized: {:?}", game.component_db);
    let mut game = game;

    let mut last_delta = std::time::Instant::now();

    loop {
        let now = std::time::Instant::now();
        let delta = now.duration_since(last_delta);
        last_delta = now;

        let update_result = game.update(delta.as_secs_f64());
        match update_result {
            Some(()) => {}
            None => break,
        }

        game.render()?;
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

/// Drops all given values.
#[macro_export]
macro_rules! drop_all {
    ($($val: expr),*) => {
        $(drop($val);)*
    };
}
