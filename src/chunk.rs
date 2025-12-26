use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{
    BlockPosition, ChunkPosition,
    block::{Block, BlockTextureAtlas},
    graphics::{
        CardinalDirection,
        lowlevel::{
            WgpuInstance,
            buf::{IndexBuffer, VertexBuffer},
        },
        mesh::{BlockMesh, BlockVertex},
    },
};

pub const CHUNK_SIZE: usize = 16;

#[derive(Clone, Debug)]
pub struct Chunk<'a> {
    pub data: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    pub render_state: RefCell<ChunkRenderState<'a>>,
}

impl<'a> Chunk<'a> {
    pub fn empty(wgpu: Rc<WgpuInstance<'a>>) -> Self {
        Self {
            data: [[[Block::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            render_state: RefCell::new(ChunkRenderState::new(wgpu.clone())),
        }
    }

    /// Inspects a block at the given world position.
    /// This can be in the range of [-1, CHUNK_SIZE + 1], where out-of-bounds positions return the neighboring chunk's block. (in the future)
    pub fn inspect_block(&self, position: BlockPosition) -> Block {
        if position.0 < 0
            || position.0 >= CHUNK_SIZE as i64
            || position.1 < 0
            || position.1 >= CHUNK_SIZE as i64
            || position.2 < 0
            || position.2 >= CHUNK_SIZE as i64
        {
            return Block::Air; // Out of bounds for now
        }
        self.data[position.0 as usize][position.1 as usize][position.2 as usize]
    }
}

impl<'a> std::ops::Deref for Chunk<'a> {
    type Target = [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a> std::ops::Index<(usize, usize, usize)> for Chunk<'a> {
    type Output = Block;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        &self.data[index.0][index.1][index.2]
    }
}

impl<'a> std::ops::IndexMut<(usize, usize, usize)> for Chunk<'a> {
    fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
        &mut self.data[index.0][index.1][index.2]
    }
}

/// Render state for a chunk.
#[derive(Debug, Clone)]
pub struct ChunkRenderState<'a> {
    block_mesh: Option<BlockMesh>,
    buffers: Option<(VertexBuffer<BlockVertex>, IndexBuffer<u16>)>,
    wgpu: Rc<WgpuInstance<'a>>,
}

impl<'a> ChunkRenderState<'a> {
    pub fn new(wgpu: Rc<WgpuInstance<'a>>) -> Self {
        Self {
            block_mesh: None,
            buffers: None,
            wgpu,
        }
    }

    /// Generates the mesh for the `chunk` `at`
    pub fn generate_mesh(
        &mut self,
        chunk: &Chunk<'a>,
        at: ChunkPosition,
        with: &BlockTextureAtlas,
    ) -> &BlockMesh {
        let mut mesh = BlockMesh::empty();

        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    let block = chunk.data[x][y][z];
                    let true_pos = (
                        x as i64 + (at.0 * CHUNK_SIZE as i64),
                        y as i64 + (at.1 * CHUNK_SIZE as i64),
                        z as i64 + (at.2 * CHUNK_SIZE as i64),
                    );
                    if block != Block::Air {
                        CardinalDirection::iter().for_each(|dir| {
                            // For now, were just going to assume that out-of-bounds blocks are air.
                            // This is a bigger problem in this engine since chunks are only 16x16x16, rather than 16x256x16.
                            if !chunk.inspect_block(dir.offset_pos(true_pos)).is_solid() {
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
    /// Returns the vertex and index buffers for the chunk.
    pub fn buffers(&self) -> (VertexBuffer<BlockVertex>, IndexBuffer<u16>) {
        let mesh = self.block_mesh.as_ref().expect("Mesh not generated");

        if let Some((vbuf, ibuf)) = &self.buffers {
            return (vbuf.clone(), ibuf.clone());
        }

        let vertex_buffer = self.wgpu.vertex_buffer::<BlockVertex>(
            bytemuck::cast_slice::<_, BlockVertex>(mesh.vertices()),
            Some("Chunk Vertex Buffer"),
        );

        let index_buffer = self.wgpu.index_buffer::<u16>(
            bytemuck::cast_slice::<_, u16>(mesh.indices()),
            Some("Chunk Index Buffer"),
        );

        (vertex_buffer, index_buffer)
    }
}
