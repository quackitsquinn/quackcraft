pub mod camera;
pub mod image;
pub mod lowlevel;
pub mod mesh;
pub mod model;

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
    pub fn to_vector(&self) -> glam::Vec3 {
        match self {
            CardinalDirection::North => glam::Vec3::new(0.0, 0.0, -1.0),
            CardinalDirection::South => glam::Vec3::new(0.0, 0.0, 1.0),
            CardinalDirection::East => glam::Vec3::new(1.0, 0.0, 0.0),
            CardinalDirection::West => glam::Vec3::new(-1.0, 0.0, 0.0),
            CardinalDirection::Up => glam::Vec3::new(0.0, 1.0, 0.0),
            CardinalDirection::Down => glam::Vec3::new(0.0, -1.0, 0.0),
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
