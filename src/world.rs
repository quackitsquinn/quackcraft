use std::{cell::RefCell, collections::HashMap};

use crate::{
    BlockPosition,
    chunk::Chunk,
    graphics::{
        Wgpu,
        lowlevel::buf::{IndexBuffer, VertexBuffer},
        mesh::BlockVertex,
    },
};

pub struct World<'a> {
    pub chunks: HashMap<BlockPosition, Chunk<'a>>,
    pub render_state: RefCell<WorldRenderState<'a>>,
}

impl<'a> World<'a> {
    /// Creates an empty World.
    pub fn empty(wgpu: Wgpu<'a>) -> Self {
        Self {
            chunks: HashMap::new(),
            render_state: RefCell::new(WorldRenderState::new(wgpu)),
        }
    }

    /// Creates a new World from the given chunks.
    pub fn new(chunks: Vec<((i64, i64, i64), Chunk<'a>)>, wgpu: Wgpu<'a>) -> Self {
        Self {
            chunks: chunks.into_iter().collect(),
            render_state: RefCell::new(WorldRenderState::new(wgpu)),
        }
    }

    /// Inserts a chunk at the given position.
    pub fn push_chunk(&mut self, position: BlockPosition, chunk: Chunk<'a>) {
        self.chunks.insert(position, chunk);
    }
}

pub struct WorldRenderState<'a> {
    pub wgpu: Wgpu<'a>,
    buffers: Option<Vec<(VertexBuffer<BlockVertex>, IndexBuffer<u16>)>>,
}

impl<'a> WorldRenderState<'a> {
    pub fn new(wgpu: Wgpu<'a>) -> Self {
        Self {
            wgpu,
            buffers: None,
        }
    }

    /// Generates the mesh for all chunks in the world.
    pub fn generate_mesh(&mut self, world: &World<'a>) {
        let mut buffers = Vec::new();

        for (pos, chunk) in world.chunks.iter() {
            let mut render_state = chunk.render_state.borrow_mut();
            render_state.generate_mesh(chunk, *pos);
            buffers.push(render_state.buffers());
        }

        self.buffers = Some(buffers);
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass<'a>) {
        if let Some(buffers) = &self.buffers {
            for (vbuf, ibuf) in buffers.iter() {
                render_pass.set_vertex_buffer(0, vbuf.buffer().slice(..));
                render_pass.set_index_buffer(ibuf.buffer().slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..ibuf.count() as u32, 0, 0..1);
            }
        }
    }
}
