use std::{cell::RefCell, collections::HashMap};

use log::info;

use crate::{
    BlockPosition, GameRef, GameState,
    block::{self, Block},
    chunk::Chunk,
    coords::bp,
    debug::{self, DebugProvider},
    graphics::{
        CardinalDirection, Wgpu,
        lowlevel::buf::{IndexBuffer, VertexBuffer},
        mesh::{BlockMesh, BlockVertex},
        render::RenderState,
    },
    resource::{ImmutableResource, Resource},
};

pub struct World {
    pub chunks: HashMap<BlockPosition, Resource<Chunk>>,
    pub render_state: RefCell<WorldRenderState>,
    debug_state: Resource<WorldDebugState>,
}

impl World {
    /// Creates an empty World.
    pub fn empty(resource_state: GameRef) -> Self {
        Self {
            chunks: HashMap::new(),
            render_state: RefCell::new(WorldRenderState::new(resource_state.clone())),
            debug_state: WorldDebugState::new(&resource_state).into(),
        }
    }

    /// Creates a new World from the given chunks.
    pub fn new(chunks: Vec<((i64, i64, i64), Chunk)>, resource_state: GameRef) -> Self {
        Self {
            chunks: chunks
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
            render_state: RefCell::new(WorldRenderState::new(resource_state.clone())),
            debug_state: WorldDebugState::new(&resource_state).into(),
        }
    }

    /// Creates a test world with some simple terrain.
    pub fn test(wgpu: GameRef) -> Self {
        let mut world = Self::empty(wgpu.clone());
        for x in 0..5 {
            for z in 0..5 {
                let mut chunk = Chunk::empty(wgpu.clone());
                for i in 0..16 {
                    for j in 0..16 {
                        if (i + j) % 2 == 0 {
                            chunk.data[i][15][j] = crate::Block::OakWood;
                        } else {
                            chunk.data[i][14][j] = crate::Block::OakLeaves;
                        }
                        chunk.data[i][3][j] = crate::Block::Grass;
                        chunk.data[i][2][j] = crate::Block::Dirt;
                        chunk.data[i][1][j] = crate::Block::Dirt;
                        chunk.data[i][0][j] = crate::Block::Stone;
                    }
                }
                world.push_chunk(bp(x, 0, z), chunk.clone());
                world.push_chunk(bp(x, 1, z), chunk);
            }
        }

        world
    }

    /// Creates a test world with a single block of the given type.
    pub fn single(resource_state: GameRef, block: Block) -> Self {
        let chunk = {
            let mut chunk = Chunk::empty(resource_state.clone());
            chunk.data[8][8][8] = block;
            chunk
        };
        let mut chunks = HashMap::new();
        chunks.insert(bp(0, 0, 0), chunk.into());
        Self {
            chunks,
            render_state: RefCell::new(WorldRenderState::new(resource_state.clone())),
            debug_state: WorldDebugState::new(&resource_state).into(),
        }
    }

    /// Inserts a chunk at the given position.
    pub fn push_chunk(&mut self, position: BlockPosition, chunk: Chunk) {
        self.chunks.insert(position, chunk.into());
    }

    /// Populates neighbor references for all chunks in the world.
    /// TODO: populate_neighbors(pos: ChunkPosition)
    pub fn populate_neighbors(&mut self) {
        for (pos, chunk) in self.chunks.iter() {
            CardinalDirection::iter().for_each(|dir| {
                let neighbor_pos = pos.offset(dir);
                if let Some(neighbor) = self.chunks.get(&neighbor_pos) {
                    chunk.get_mut().set_neighbor(dir, Some(neighbor.clone()));
                }
            });
        }
    }
}

pub struct WorldRenderState {
    pub game_state: GameRef,
    meshes: HashMap<BlockPosition, BlockMesh>,
    buffers: Option<Vec<(VertexBuffer<BlockVertex>, IndexBuffer<u16>)>>,
}

impl WorldRenderState {
    pub fn new(game_state: GameRef) -> Self {
        Self {
            game_state,
            meshes: HashMap::new(),
            buffers: None,
        }
    }

    /// Generates the mesh for all chunks in the world.
    pub fn generate_mesh(&mut self, world: &World, with: &crate::block::BlockTextureAtlas) {
        // Ok so, rather than generate area^3, we merge all buffers in y axis only.
        let mut meshes = HashMap::new();
        let render_state = &self.game_state.render_state();

        for (pos, chunk) in world.chunks.iter() {
            let chunk = chunk.get();
            let mut render_state = chunk.render_state.borrow_mut();
            let mesh = render_state.generate_mesh(&chunk, *pos, with);
            meshes
                .entry(*pos)
                .and_modify(|f: &mut BlockMesh| *f = mesh.clone())
                .or_insert_with(|| mesh.clone());
        }

        self.meshes = meshes;

        let mut total_faces = 0;
        let buffers = self
            .meshes
            .values()
            .map(|mesh| {
                total_faces += mesh.face_count();
                mesh.create_buffers(render_state)
            })
            .collect();

        world.debug_state.get_mut().update_face_count(total_faces);

        self.buffers = Some(buffers);
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        if let Some(buffers) = &self.buffers {
            for (vbuf, ibuf) in buffers.iter() {
                render_pass.set_vertex_buffer(0, vbuf.buffer().slice(..));
                render_pass.set_index_buffer(ibuf.buffer().slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..ibuf.count() as u32, 0, 0..1);
            }
        }
    }
}

struct WorldDebugState {
    /// Current number of faces being rendered.
    face_count: usize,
    /// Debug provider for the face count.
    face_count_provider: DebugProvider,
}

impl WorldDebugState {
    pub fn new(game_state: &ImmutableResource<GameState>) -> Self {
        let mut render_state = game_state.render_state_mut();
        let face_count_provider = render_state.debug_renderer.add_statistic("Face Count", "0");
        Self {
            face_count: 0,
            face_count_provider,
        }
    }

    pub fn update_face_count(&mut self, new_count: usize) {
        self.face_count = new_count;
        self.face_count_provider.update_value(self.face_count);
    }
}
