use engine::graphics::{CardinalDirection, textures::TextureHandle};

// TODO: fancy optimizations for blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Block {
    Air = 0,
    Dirt = 1,
    Stone = 2,
    Grass = 3,
    OakWood = 4,
    OakLeaves = 5,
}

impl Block {
    pub const MAX_DISCRIMINANT: u8 = 5;

    pub fn is_solid(&self) -> bool {
        !matches!(self, Block::Air | Block::OakLeaves)
    }

    /// Gets the block ID for the given texture handle and direction, if applicable.
    pub fn id_from(&self, handle: TextureHandle, direction: CardinalDirection) -> u32 {
        match self {
            Block::Grass => match direction {
                CardinalDirection::Up => handle.layer(1),
                CardinalDirection::Down => handle.layer(2),
                _ => handle.layer(0),
            },
            Block::OakWood => match direction {
                CardinalDirection::Up | CardinalDirection::Down => handle.layer(1),
                _ => handle.layer(0),
            },
            _ => handle.layer(0),
        }
    }
}
