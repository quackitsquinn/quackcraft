use log::warn;

use crate::{BlockPosition, world::Block};

use engine::{component::ComponentStoreHandle, graphics::CardinalDirection, resource::Resource};

pub const CHUNK_SIZE: usize = 16;

#[derive(Clone, Debug)]
pub struct Chunk {
    pub data: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    neighbors: [Option<Resource<Chunk>>; 6],
}

impl Chunk {
    pub fn empty(_state: ComponentStoreHandle) -> Self {
        Self {
            data: [[[Block::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            neighbors: [None, None, None, None, None, None],
        }
    }

    pub fn set_neighbor(
        &mut self,
        direction: CardinalDirection,
        neighbor: Option<Resource<Chunk>>,
    ) {
        self.neighbors[direction as usize] = neighbor;
    }

    /// Inspects a block at the given local chunk position.
    pub fn inspect_block_exact(&self, position: BlockPosition) -> Block {
        self.data[position.0 as usize][position.1 as usize][position.2 as usize]
    }

    /// Inspects a block at the given world position + direction.
    pub fn inspect_block(&self, base: BlockPosition, direction: CardinalDirection) -> Block {
        // We need to return the block (if present) in the given direction from the base position.
        let true_pos = base.offset(direction);
        if true_pos.all(|c| c > CHUNK_SIZE as i64 * 2) {
            warn!(
                "Inspecting block at very large positive position {:?}",
                true_pos
            );
            return Block::Air;
        }
        let local_pos = true_pos.chunk_normalize();
        if true_pos == local_pos {
            // Still in this chunk
            self.inspect_block_exact(local_pos)
        } else {
            // In a neighbor chunk
            if let Some(neighbor) = &self.neighbors[direction as usize] {
                // We now just need to make sure that the true pos was only offset by one chunk in the given direction.
                neighbor.get().inspect_block_exact(local_pos)
            } else {
                Block::Air // Out of bounds, assume air
            }
        }
    }
}

impl std::ops::Deref for Chunk {
    type Target = [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl std::ops::Index<(usize, usize, usize)> for Chunk {
    type Output = Block;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        &self.data[index.0][index.1][index.2]
    }
}

impl std::ops::IndexMut<(usize, usize, usize)> for Chunk {
    fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
        &mut self.data[index.0][index.1][index.2]
    }
}
