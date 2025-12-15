use std::rc::{Rc, Weak};

use crate::{block::Block, graphics::lowlevel::WgpuInstance};

#[derive(Clone, Debug)]
pub struct Chunk<'a> {
    data: [[[Block; 16]; 16]; 16],
    wgpu: Rc<WgpuInstance<'a>>,
    this: Weak<Chunk<'a>>,
}

impl<'a> Chunk<'a> {
    pub fn empty(wgpu: Rc<WgpuInstance<'a>>) -> Rc<Self> {
        Rc::new_cyclic(|weak| Self {
            data: [[[Block::Air; 16]; 16]; 16],
            this: weak.clone(),
            wgpu: wgpu.clone(),
        })
    }
}

impl<'a> std::ops::Deref for Chunk<'a> {
    type Target = [[[Block; 16]; 16]; 16];

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

/// Render state for a chunk.
struct ChunkRenderState {}
