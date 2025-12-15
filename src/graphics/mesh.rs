use bytemuck::{Pod, Zeroable};
use glam::Vec3;

use crate::{
    BlockPosition,
    block::Block,
    graphics::{CardinalDirection, lowlevel::buf::VertexLayout},
};
#[derive(Clone, Debug)]
pub struct BlockMesh {
    vertices: Vec<BlockVertex>,
    indices: Vec<u16>,
}

pub const FACE_TABLE: [[[f32; 3]; 4]; 6] = [
    // +X
    [
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 0.0],
        [1.0, 0.0, 0.0],
    ],
    // -X
    [
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0],
    ],
    // +Y
    [
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
    ],
    // -Y
    [
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
    ],
    // +Z
    [
        [0.0, 1.0, 1.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
    ],
    // -Z
    [
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0],
    ],
];

pub const FACE_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

impl BlockMesh {
    pub fn new(vertices: Vec<BlockVertex>, indices: Vec<u16>) -> Self {
        Self { vertices, indices }
    }

    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Checks if the mesh already contains the given vertex.
    /// Returns the index of the vertex if it exists.
    pub fn contains_vertex(&self, vertex: &BlockVertex) -> Option<usize> {
        self.vertices.iter().position(|v| v.round_eq(vertex))
    }

    /// Pushes a vertex to the mesh and returns its index.
    pub fn push_vertex(&mut self, vertex: BlockVertex) -> u16 {
        self.vertices.push(vertex);
        (self.vertices.len() - 1) as u16
    }

    /// Emits a face for the given block position in the given direction.
    pub fn emit_face(
        &mut self,
        block: &Block,
        position: BlockPosition,
        direction: CardinalDirection,
    ) {
        let mut face = FACE_TABLE[direction as usize];

        let mut face_indices = [0; 6];

        for (i, face) in face.iter_mut().enumerate() {
            face[0] += position.0 as f32;
            face[1] += position.1 as f32;
            face[2] += position.2 as f32;

            let color = block.col();
            let vertex = BlockVertex {
                position: *face,
                color: [color.x, color.y, color.z, color.w],
            };

            let vertex_index = match self.contains_vertex(&vertex) {
                Some(index) => index as u16,
                None => self.push_vertex(vertex),
            };

            face_indices[i] = vertex_index;
        }

        FACE_INDICES.iter().for_each(|&i| {
            self.indices.push(face_indices[i as usize]);
        });
    }

    pub fn vertices(&self) -> &Vec<BlockVertex> {
        &self.vertices
    }

    pub fn indices(&self) -> &Vec<u16> {
        &self.indices
    }
}

#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct BlockVertex {
    position: [f32; 3],
    color: [f32; 4],
}

impl BlockVertex {
    pub fn round_eq(&self, other: &Self) -> bool {
        let pos_eq = self
            .position
            .iter()
            .zip(other.position.iter())
            .all(|(a, b)| (a - b).abs() < f32::EPSILON);
        let color_eq = self
            .color
            .iter()
            .zip(other.color.iter())
            .all(|(a, b)| (a - b).abs() < f32::EPSILON);
        pos_eq && color_eq
    }
}

impl PartialEq for BlockVertex {
    fn eq(&self, other: &Self) -> bool {
        self.round_eq(other)
    }
}

unsafe impl VertexLayout for BlockVertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<BlockVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x3, // position
            1 => Float32x4, // color
        ],
    };
}
