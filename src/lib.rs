use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    rc::{Rc, Weak},
    sync::Arc,
};

use glam::Vec3;
use glfw::{Action, Key, WindowEvent};
use log::info;
use wgpu::{Color, PrimitiveState};

use crate::{
    assets::AssetStore,
    block::{Block, BlockTextureAtlas},
    debug::{DebugProvider, DebugRenderer},
    graphics::{
        Wgpu,
        lowlevel::{WgpuInstance, buf::VertexLayout},
        mesh::BlockVertex,
        postprocess::PostProcessingPass,
        render::RenderState,
        textures::TextureCollection,
    },
    input::{camera::CameraController, keyboard::Keyboard},
    resource::{ImmutableResource, Resource, WeakImmutableResource, WeakResource},
    window::GlfwWindow,
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
/// A reference-counted game state.
pub type GameRef = ImmutableResource<GameState>;

mod assets;
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
#[derive(Debug)] // FYI: This is the top layer of the entire game state. This might create a massive debug output.
pub struct GameState {
    keyboard: RefCell<Keyboard>,
    render_state: RefCell<RenderState>,
    assets: RefCell<AssetStore>,
    delta_time: Cell<f32>,
    // TODO: Maybe HashMap<TypeId, Resource<dyn Any>> for generic resources?
    this: WeakImmutableResource<GameState>,
}

impl GameState {
    /// Creates a new game instance.
    pub fn new(render_state: RenderState) -> anyhow::Result<ImmutableResource<GameState>> {
        let keyboard = RefCell::new(Keyboard::new());

        let game = ImmutableResource::new_cyclic(|w| GameState {
            render_state: RefCell::new(render_state),
            assets: RefCell::new(AssetStore {}),
            keyboard,
            delta_time: Cell::new(0.0),
            this: w.clone(),
        });

        Ok(game)
    }

    fn update_camera(&self, _frame: u64) {
        // This is why camera needs to own the bindgroup/layout
        let mut render_state = self.render_state.borrow_mut();
        let mut camera = render_state.camera.0.get_mut();
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
        drop(camera);

        if keyboard.is_key_pressed(Key::F3) {
            render_state.debug_renderer.toggle();
        }

        if keyboard.is_key_pressed(Key::Escape) {
            render_state.window.get_mut().toggle_mouse_mode();
        }
    }

    pub fn render(&self) -> anyhow::Result<()> {
        let frame_start = std::time::Instant::now();
        // TODO: actually render stuff
        let frame_end = std::time::Instant::now();
        let delta = frame_end.duration_since(frame_start).as_secs_f32();
        self.delta_time.set(delta);
        Ok(())
    }
}

impl ImmutableResource<GameState> {
    /// Returns a reference to the game's render state.
    pub fn render_state(&self) -> Ref<'_, RenderState> {
        self.render_state.borrow()
    }

    /// Returns a mutable reference to the game's render state.
    pub fn render_state_mut(&self) -> RefMut<'_, RenderState> {
        self.render_state.borrow_mut()
    }

    /// Returns a reference to the game's keyboard state.
    pub fn keyboard(&self) -> Ref<'_, Keyboard> {
        self.keyboard.borrow()
    }

    /// Returns a mutable reference to the game's keyboard state.
    pub fn keyboard_mut(&self) -> RefMut<'_, Keyboard> {
        self.keyboard.borrow_mut()
    }

    /// Returns a reference to the game's asset store.
    pub fn assets(&self) -> Ref<'_, AssetStore> {
        self.assets.borrow()
    }

    /// Returns a mutable reference to the game's asset store.
    pub fn assets_mut(&self) -> RefMut<'_, AssetStore> {
        self.assets.borrow_mut()
    }

    /// Returns the game's render state.
    pub fn should_close(&self) -> bool {
        self.render_state().window.get().should_close()
    }
}

pub fn run_game() -> anyhow::Result<()> {
    info!("Starting quackcraft");
    let render_state = smol::block_on(RenderState::new("Quackcraft", (1280, 720)))?;
    let game = GameState::new(render_state)?;
    while !game.should_close() {
        let render_state = game.render_state();
        let mut window = render_state.window.get_mut();
        window.poll_events();
        let mut keyboard = game.keyboard_mut();
        keyboard.update_keys();
        while let Some((_, event)) = window.event_receiver.receive() {
            match event {
                WindowEvent::Close => break,
                WindowEvent::Size(x, y) => {
                    render_state.wgpu.resize((x, y));
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
        drop(window);
        drop(render_state);
        drop(keyboard);
        game.update_camera(0);
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
