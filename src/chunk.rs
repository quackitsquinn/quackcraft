use std::{cell::RefCell, rc::Rc};

use log::{info, warn};

use crate::{
    BlockPosition, ChunkPosition,
    block::{Block, BlockTextureAtlas},
    coords::bp,
    graphics::{
        CardinalDirection, Wgpu,
        lowlevel::{
            WgpuInstance,
            buf::{IndexBuffer, VertexBuffer},
        },
        mesh::{BlockMesh, BlockVertex},
    },
    resource::Resource,
};

pub const CHUNK_SIZE: usize = 16;

#[derive(Clone, Debug)]
pub struct Chunk {
    pub data: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    neighbors: [Option<Resource<Chunk>>; 6],
    pub render_state: RefCell<ChunkRenderState>,
}

impl Chunk {
    pub fn empty(wgpu: Rc<WgpuInstance>) -> Self {
        Self {
            data: [[[Block::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            neighbors: [None, None, None, None, None, None],
            render_state: RefCell::new(ChunkRenderState::new(wgpu.clone())),
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

/// Render state for a chunk.
#[derive(Debug, Clone)]
pub struct ChunkRenderState {
    block_mesh: Option<BlockMesh>,
    buffers: Option<(VertexBuffer<BlockVertex>, IndexBuffer<u16>)>,
    wgpu: Wgpu,
}

impl ChunkRenderState {
    pub fn new(wgpu: Rc<WgpuInstance>) -> Self {
        Self {
            block_mesh: None,
            buffers: None,
            wgpu,
        }
    }

    /// Generates the mesh for the `chunk` `at`
    pub fn generate_mesh(
        &mut self,
        chunk: &Chunk,
        at: ChunkPosition,
        with: &BlockTextureAtlas,
    ) -> &BlockMesh {
        let mut mesh = BlockMesh::empty();

        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    let block = chunk.data[x][y][z];
                    let true_pos = bp(
                        x as i64 + (at.0 * CHUNK_SIZE as i64),
                        y as i64 + (at.1 * CHUNK_SIZE as i64),
                        z as i64 + (at.2 * CHUNK_SIZE as i64),
                    );
                    let rel_pos = bp(x as i64, y as i64, z as i64);
                    if block != Block::Air {
                        // TODO.. in the probably distant future: greedy meshing
                        CardinalDirection::iter().for_each(|dir| {
                            // For now, were just going to assume that out-of-bounds blocks are air.
                            // This is a bigger problem in this engine since chunks are only 16x16x16, rather than 16x256x16.
                            if !chunk.inspect_block(rel_pos, dir).is_solid() {
                                mesh.emit_face(&with.face_texture_index(block, dir), true_pos, dir);
                            }
                        });
                    }
                }
            }
        }

        self.block_mesh = Some(mesh);
        self.buffers = None; // Invalidate buffers
        self.block_mesh.as_ref().unwrap()
    }

    /// Generates the vertex and index buffers for the current mesh, if not already generated.
    pub fn generate_buffers(&mut self) -> (&VertexBuffer<BlockVertex>, &IndexBuffer<u16>) {
        if self.buffers.is_none() {
            let mesh = self
                .block_mesh
                .as_ref()
                .expect("Mesh must be generated before buffers");
            self.buffers = Some(mesh.create_buffers(&self.wgpu));
        }
        let (vb, ib) = self.buffers.as_ref().unwrap();
        (vb, ib)
    }
}
