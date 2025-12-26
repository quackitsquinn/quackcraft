use glam::Vec4;

use crate::graphics::textures::TextureHandle;

// TODO: fancy optimizations for blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Block {
    Air = 0,
    Dirt = 1,
    Stone = 2,
}

impl Block {
    pub fn is_solid(&self) -> bool {
        // So, there is an interesting story behind this function.
        // I was really curious about if a match statement would be faster than
        // a simple comparison, so I wrote both versions and put them through godbolt.
        // Surprisingly, the match statement produced slightly more optimized assembly.
        // I was curious about what happened when optimizations are enabled, so I checked that too.
        // and, incredibly, one of the functions was completely optimized away and replaced with a single comparison instruction!
        // So, I decided to go with the match statement for now, as it seems to be the most efficient in both cases.
        !matches!(self, Block::Air)
    }

    pub fn color(&self) -> Vec4 {
        match self {
            Block::Air => Vec4::new(0.0, 0.0, 0.0, 0.0),
            Block::Dirt => Vec4::new(0.59, 0.29, 0.0, 1.0),
            Block::Stone => Vec4::new(0.5, 0.5, 0.5, 1.0),
        }
    }
}

pub struct BlockTextureAtlas {
    textures: [TextureHandle; 256],
}

impl BlockTextureAtlas {
    pub fn new(default_texture: TextureHandle) -> Self {
        Self {
            textures: [default_texture; 256],
        }
    }

    pub fn set_texture_handle(&mut self, block: Block, handle: TextureHandle) {
        self.textures[block as usize] = handle;
    }

    pub fn get_texture_handle(&self, block: Block) -> &TextureHandle {
        &self.textures[block as usize]
    }
}
