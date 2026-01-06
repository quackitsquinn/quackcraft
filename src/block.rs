use engine::graphics::textures::TextureHandle;

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

// TODO: The texture atlas being just a texture handle that you increment for unique side textures is... not great.
// I think that it's a good solution in spirit, but the current lack of abstraction makes it super error-prone.
// This can probably be made into just an actual TextureHandle struct that handles all of this internally.
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

    pub fn get_base_handle(&self, block: Block) -> TextureHandle {
        self.textures[block as usize]
    }

    /// Returns the texture index for the given face of the block.
    pub fn face_texture_index(
        &self,
        block: Block,
        direction: engine::graphics::CardinalDirection,
    ) -> TextureHandle {
        match block {
            // Generate column like textures for grass and leaves
            Block::Grass | Block::OakWood => {
                let column_base = self.get_base_handle(block);
                // Grass has different textures for top, bottom, and sides
                match direction {
                    engine::graphics::CardinalDirection::Up => column_base, // Top texture
                    engine::graphics::CardinalDirection::Down => {
                        // If the block overrides the bottom texture, use that.
                        // Otherwise, use the default bottom texture.
                        if let Some(bottom) = block.bottom_texture() {
                            self.get_base_handle(bottom)
                        } else {
                            column_base
                        }
                    }
                    _ => column_base + 1, // Side texture
                }
            }
            _ => self.get_base_handle(block),
        }
    }
}
