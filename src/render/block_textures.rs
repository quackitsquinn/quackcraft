use engine::graphics::textures::{TextureCollection, TextureHandle};

use crate::world::{Block, block};

pub struct BlockTextureAtlas {
    handles: [TextureHandle; Block::MAX_DISCRIMINANT as usize + 1],
}

impl BlockTextureAtlas {
    /// Creates a new BlockTextureAtlas from the given TextureCollection.
    pub fn new() -> Self {
        Self {
            handles: [TextureHandle::null(); Block::MAX_DISCRIMINANT as usize + 1],
        }
    }

    /// Gets the texture handle for the given block.
    pub fn get_texture_handle(&self, block: Block) -> TextureHandle {
        self.handles[block as usize]
    }

    /// Sets the texture handle for the given block.
    pub fn set_texture_handle(&mut self, block: Block, handle: TextureHandle) {
        self.handles[block as usize] = handle;
    }

    /// Gets the texture index for the given block and direction.
    pub fn texture_index(
        &self,
        block: Block,
        direction: engine::graphics::CardinalDirection,
    ) -> u32 {
        block.id_from(self.get_texture_handle(block), direction)
    }
}

impl Default for BlockTextureAtlas {
    fn default() -> Self {
        Self::new()
    }
}
