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
    pub fn is_solid(&self) -> bool {
        !matches!(self, Block::Air | Block::OakLeaves)
    }

    pub fn bottom_texture(&self) -> Option<Block> {
        match self {
            Block::Grass => Some(Block::Dirt),
            _ => None,
        }
    }
}
