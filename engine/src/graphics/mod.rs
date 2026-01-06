pub mod callback;
pub mod camera;
pub mod image;
pub mod lowlevel;
pub mod pipeline;
pub mod textures;

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
