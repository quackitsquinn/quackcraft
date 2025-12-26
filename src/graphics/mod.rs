use std::rc::Rc;

use glam::Vec3;

use crate::BlockPosition;

pub mod callback;
pub mod camera;
pub mod image;
pub mod lowlevel;
pub mod mesh;
pub mod model;
pub mod textures;

/// A reference-counted WGPU instance.
// TODO: WgpuInstance should be renamed and probably placed in a Rc wrapper here.
pub type Wgpu<'a> = Rc<lowlevel::WgpuInstance<'a>>;

/// Cardinal directions in 3D space.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CardinalDirection {
    /// East direction (+X axis)
    East,
    /// West direction (-X axis)
    West,
    /// Up direction (+Y axis)
    Up,
    /// Down direction (-Y axis)
    Down,
    /// South direction (+Z axis)
    South,
    /// North direction (-Z axis)
    North,
}

impl CardinalDirection {
    /// Returns the normal vector corresponding to the cardinal direction.
    pub fn normal(&self) -> glam::Vec3 {
        match self {
            CardinalDirection::North => glam::Vec3::new(0.0, 0.0, -1.0),
            CardinalDirection::South => glam::Vec3::new(0.0, 0.0, 1.0),
            CardinalDirection::East => glam::Vec3::new(1.0, 0.0, 0.0),
            CardinalDirection::West => glam::Vec3::new(-1.0, 0.0, 0.0),
            CardinalDirection::Up => glam::Vec3::new(0.0, 1.0, 0.0),
            CardinalDirection::Down => glam::Vec3::new(0.0, -1.0, 0.0),
        }
    }

    /// Returns the normal vector as i64 components.
    pub fn normal_i64(&self) -> (i64, i64, i64) {
        match self {
            CardinalDirection::North => (0, 0, -1),
            CardinalDirection::South => (0, 0, 1),
            CardinalDirection::East => (1, 0, 0),
            CardinalDirection::West => (-1, 0, 0),
            CardinalDirection::Up => (0, 1, 0),
            CardinalDirection::Down => (0, -1, 0),
        }
    }

    /// Offsets the given block position by one unit in the direction of the cardinal direction.
    pub fn offset_pos(&self, pos: BlockPosition) -> BlockPosition {
        let (dx, dy, dz) = self.normal_i64();
        (pos.0 + dx, pos.1 + dy, pos.2 + dz)
    }

    pub fn iter() -> impl Iterator<Item = CardinalDirection> {
        [
            CardinalDirection::North,
            CardinalDirection::South,
            CardinalDirection::East,
            CardinalDirection::West,
            CardinalDirection::Up,
            CardinalDirection::Down,
        ]
        .into_iter()
    }
}
