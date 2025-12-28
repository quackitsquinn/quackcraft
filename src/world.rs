use std::{cell::RefCell, collections::HashMap};

use crate::{
    BlockPosition,
    block::Block,
    chunk::Chunk,
    debug::{self, DebugProvider},
    graphics::{
        Wgpu,
        lowlevel::buf::{IndexBuffer, VertexBuffer},
        mesh::{BlockMesh, BlockVertex},
    },
};

pub struct World<'a> {
    pub chunks: HashMap<BlockPosition, Chunk<'a>>,
    pub render_state: RefCell<WorldRenderState<'a>>,
    face_count: Option<DebugProvider>,
}

impl<'a> World<'a> {
    /// Creates an empty World.
    pub fn empty(wgpu: Wgpu<'a>) -> Self {
        Self {
            chunks: HashMap::new(),
            render_state: RefCell::new(WorldRenderState::new(wgpu)),
            face_count: None,
        }
    }

    /// Creates a new World from the given chunks.
    pub fn new(chunks: Vec<((i64, i64, i64), Chunk<'a>)>, wgpu: Wgpu<'a>) -> Self {
        Self {
            chunks: chunks.into_iter().collect(),
            render_state: RefCell::new(WorldRenderState::new(wgpu)),
            face_count: None,
        }
    }

    /// Creates debug providers for this world.
    pub fn create_debug_providers<'b>(&mut self, debug_renderer: &mut debug::DebugRenderer<'b>) {
        let face_count = debug_renderer.add_statistic("Face Count", "0");
        self.face_count = Some(face_count);
    }

    /// Creates a test world with some simple terrain.
    pub fn test(wgpu: Wgpu<'a>) -> Self {
        let mut chunks = HashMap::new();
        for x in 0..5 {
            for z in 0..5 {
                let mut chunk = Chunk::empty(wgpu.clone());
                for i in 0..16 {
                    for j in 0..16 {
                        chunk.data[i][3][j] = crate::Block::Grass;
                        chunk.data[i][2][j] = crate::Block::Dirt;
                        chunk.data[i][2][j] = crate::Block::Dirt;
                        chunk.data[i][1][j] = crate::Block::Stone;
                    }
                }
                chunks.insert((x, 0, z), chunk);
            }
        }

        Self {
            chunks,
            render_state: RefCell::new(WorldRenderState::new(wgpu)),
            face_count: None,
        }
    }

    /// Creates a test world with a single block of the given type.
    pub fn single(wgpu: Wgpu<'a>, block: Block) -> Self {
        let chunk = {
            let mut chunk = Chunk::empty(wgpu.clone());
            chunk.data[8][8][8] = block;
            chunk
        };
        let mut chunks = HashMap::new();
        chunks.insert((0, 0, 0), chunk);
        Self {
            chunks,
            render_state: RefCell::new(WorldRenderState::new(wgpu)),
            face_count: None,
        }
    }

    /// Inserts a chunk at the given position.
    pub fn push_chunk(&mut self, position: BlockPosition, chunk: Chunk<'a>) {
        self.chunks.insert(position, chunk);
    }
}

pub struct WorldRenderState<'a> {
    pub wgpu: Wgpu<'a>,
    meshes: HashMap<(i64, i64), BlockMesh>,
    buffers: Option<Vec<(VertexBuffer<BlockVertex>, IndexBuffer<u16>)>>,
}

impl<'a> WorldRenderState<'a> {
    pub fn new(wgpu: Wgpu<'a>) -> Self {
        Self {
            wgpu,
            meshes: HashMap::new(),
            buffers: None,
        }
    }

    /// Generates the mesh for all chunks in the world.
    pub fn generate_mesh(&mut self, world: &World<'a>, with: &crate::block::BlockTextureAtlas) {
        // Ok so, rather than generate area^3, we merge all buffers in y axis only.
        let mut meshes = HashMap::new();

        for (pos, chunk) in world.chunks.iter() {
            let mut render_state = chunk.render_state.borrow_mut();
            let mesh = render_state.generate_mesh(chunk, *pos, with);
            meshes
                .entry((pos.0, pos.2))
                .and_modify(|f: &mut BlockMesh| f.combine(mesh))
                .or_insert_with(|| mesh.clone());
        }

        self.meshes = meshes;

        let mut total_faces = 0;
        let buffers = self
            .meshes
            .values()
            .map(|mesh| {
                total_faces += mesh.face_count();
                mesh.create_buffers(&self.wgpu)
            })
            .collect();

        if let Some(face_count) = &world.face_count {
            face_count.update_value(total_faces.to_string());
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
