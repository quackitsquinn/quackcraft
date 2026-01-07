use std::collections::HashMap;

use crate::{BlockPosition, coords::bp};

use engine::{component::ComponentStoreHandle, graphics::CardinalDirection, resource::Resource};

pub mod block;
pub mod chunk;

pub use block::Block;
pub use chunk::Chunk;

pub struct World {
    pub chunks: HashMap<BlockPosition, Resource<Chunk>>,
}

impl World {
    /// Creates an empty World.
    pub fn empty(_resource_state: &ComponentStoreHandle) -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    /// Creates a new World from the given chunks.
    pub fn new(
        chunks: Vec<((i64, i64, i64), Chunk)>,
        _resource_state: &ComponentStoreHandle,
    ) -> Self {
        Self {
            chunks: chunks
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }

    /// Creates a test world with some simple terrain.
    pub fn test(wgpu: &ComponentStoreHandle) -> Self {
        let mut world = Self::empty(wgpu);
        for x in 0..5 {
            for z in 0..5 {
                let mut chunk = Chunk::empty(wgpu.clone());
                for i in 0..16 {
                    for j in 0..16 {
                        if (i + j) % 2 == 0 {
                            chunk.data[i][15][j] = Block::OakWood;
                        } else {
                            chunk.data[i][14][j] = Block::OakLeaves;
                        }
                        chunk.data[i][3][j] = Block::Grass;
                        chunk.data[i][2][j] = Block::Dirt;
                        chunk.data[i][1][j] = Block::Dirt;
                        chunk.data[i][0][j] = Block::Stone;
                    }
                }
                world.push_chunk(bp(x, 0, z), chunk.clone());
                world.push_chunk(bp(x, 1, z), chunk);
            }
        }

        world.populate_neighbors();

        world
    }

    /// Creates a test world with a single block of the given type.
    pub fn single(resource_state: &ComponentStoreHandle, block: Block) -> Self {
        let chunk = {
            let mut chunk = Chunk::empty(resource_state.clone());
            chunk.data[8][8][8] = block;
            chunk
        };
        let mut chunks = HashMap::new();
        chunks.insert(bp(0, 0, 0), chunk.into());
        Self { chunks }
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

/// The currently active world.
pub struct ActiveWorld {
    world: Option<World>,
}

impl ActiveWorld {
    /// Creates a new, empty ActiveWorld.
    pub fn new() -> Self {
        Self { world: None }
    }

    /// Creates a new ActiveWorld with the given world.
    pub fn with_world(world: World) -> Self {
        Self { world: Some(world) }
    }

    /// Sets the active world.
    pub fn set_world(&mut self, world: World) {
        self.world = Some(world);
    }

    /// Gets a reference to the active world.
    pub fn get_world(&self) -> Option<&World> {
        self.world.as_ref()
    }

    /// Gets a mutable reference to the active world.   
    pub fn get_world_mut(&mut self) -> Option<&mut World> {
        self.world.as_mut()
    }
}
