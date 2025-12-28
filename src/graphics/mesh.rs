use bytemuck::{Pod, Zeroable};

use crate::{
    BlockPosition,
    graphics::{
        CardinalDirection, FACE_INDICES, FACE_TABLE,
        lowlevel::buf::{IndexBuffer, VertexBuffer, VertexLayout},
        textures::TextureHandle,
    },
};
#[derive(Clone, Debug)]
pub struct BlockMesh {
    vertices: Vec<BlockVertex>,
    indices: Vec<u16>,
    face_count: usize,
}

impl BlockMesh {
    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            face_count: 0,
        }
    }

    /// Pushes a vertex to the mesh and returns its index.
    pub fn push_vertex(&mut self, vertex: BlockVertex) -> u16 {
        self.vertices.push(vertex);
        (self.vertices.len() - 1) as u16
    }

    /// Emits a face for the given block position in the given direction.
    pub fn emit_face(
        &mut self,
        handle: &TextureHandle,
        position: BlockPosition,
        direction: CardinalDirection,
    ) {
        self.face_count += 1;

        let mut face = FACE_TABLE[direction as usize];

        let mut face_indices = [0; 6];

        for (i, vert) in face.iter_mut().enumerate() {
            let face = &mut vert.0;
            let tex_coords = &vert.1;
            face[0] += position.0 as f32;
            face[1] += position.1 as f32;
            face[2] += position.2 as f32;

            let vertex = BlockVertex {
                position: *face,
                tex_coord: *tex_coords,
                block_type: *handle,
            };

            face_indices[i] = self.push_vertex(vertex);
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

    /// Combines another mesh into this mesh.
    pub fn combine(&mut self, other: &BlockMesh) {
        let index_offset = self.vertices.len() as u16;

        self.vertices.extend_from_slice(&other.vertices);

        self.indices
            .extend(other.indices.iter().map(|&i| i + index_offset));

        self.face_count += other.face_count;
    }

    /// Creates the vertex and index buffers for the mesh.
    pub fn create_buffers<'a>(
        &self,
        wgpu: &crate::graphics::Wgpu<'a>,
    ) -> (VertexBuffer<BlockVertex>, IndexBuffer<u16>) {
        let vertex_buffer = wgpu.vertex_buffer::<BlockVertex>(
            bytemuck::cast_slice::<_, BlockVertex>(self.vertices()),
            Some("BlockMesh Vertex Buffer"),
        );

        let index_buffer = wgpu.index_buffer::<u16>(
            bytemuck::cast_slice::<_, u16>(self.indices()),
            Some("BlockMesh Index Buffer"),
        );

        (vertex_buffer, index_buffer)
    }

    /// Returns the number of faces in the mesh.
    pub fn face_count(&self) -> usize {
        self.face_count
    }
}

#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct BlockVertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
    block_type: u32,
}

impl BlockVertex {
    pub fn round_eq(&self, other: &Self) -> bool {
        let pos_eq = self
            .position
            .iter()
            .zip(other.position.iter())
            .all(|(a, b)| (a - b).abs() < f32::EPSILON);
        let color_eq = self
            .tex_coord
            .iter()
            .zip(other.tex_coord.iter())
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
            1 => Float32x2, // tex_coord
            2 => Uint32,    // block type
        ],
    };
}
