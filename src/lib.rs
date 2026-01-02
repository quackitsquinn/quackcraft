use std::sync::Arc;

use anyhow::Ok;
use glam::Vec3;

use crate::{
    block::Block, component::State, graphics::lowlevel::WgpuRenderer, input::keyboard::Keyboard,
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

mod assets;
mod block;
mod chunk;
mod component;
pub mod coords;
mod debug;
pub mod graphics;
mod input;
pub mod resource;
mod window;
mod world;

pub struct Game {
    component_db: State,
}

impl Game {
    pub fn new() -> anyhow::Result<Self> {
        let mut state = State::new();
        state.insert(Keyboard::new());
        let window = window::GlfwWindow::new(800, 600, "Minecraft Clone")
            .expect("Failed to create GLFW window");

        smol::block_on(WgpuRenderer::attach_to(&mut state, &window))?;

        state.insert(window);

        state.finish_initialization();

        Ok(Self {
            component_db: state,
        })
    }
}

pub fn run_game() -> anyhow::Result<()> {
    let game = Game::new()?;

    println!("Game initialized: {:?}", game.component_db);
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
