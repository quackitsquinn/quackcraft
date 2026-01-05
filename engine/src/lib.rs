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

pub mod assets;
pub mod block;
pub mod chunk;
pub mod component;
pub mod coords;
pub mod debug;
pub mod graphics;
pub mod input;
pub mod resource;
pub mod window;
pub mod world;
