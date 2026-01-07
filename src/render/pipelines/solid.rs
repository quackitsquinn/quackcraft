use std::collections::HashMap;

use engine::{
    component::{ComponentHandle, ComponentStoreHandle},
    graphics::{
        CardinalDirection,
        lowlevel::{
            WgpuRenderer,
            buf::{IndexBuffer, VertexBuffer, VertexLayout},
        },
        pipeline::{RenderPipeline, controller::PipelineKey},
    },
};
use glam::{Vec2, Vec3};
use wgpu::naga::Block;

use crate::{
    BlockPosition, FACE_INDICES, FACE_TABLE,
    coords::bp,
    world::{ActiveWorld, Chunk},
};

pub struct SolidGeometryPipeline {
    chunks: HashMap<BlockPosition, ChunkSolidRenderData>,
    world: ComponentHandle<ActiveWorld>,
    wgpu: ComponentHandle<WgpuRenderer>,
}

impl SolidGeometryPipeline {
    pub fn new(csh: &ComponentStoreHandle) -> SolidGeometryPipeline {
        Self {
            world: csh.handle_for(),
            wgpu: csh.handle_for(),
            chunks: HashMap::new(),
        }
    }

    fn create_pipeline(&self) -> wgpu::RenderPipeline {
        todo!(
            "implement some sort of PipelineBuilder system because im tired of 8 parameter functions"
        )
    }

    /// Creates initial chunk render data for all chunks in the world.
    pub fn create_initial_chunks(&mut self) {
        let world_ref = self.world.get();
        let world = world_ref.get_world().expect("no world present");

        for (chunk_coord, chunk_res) in world.chunks.iter() {
            let chunk = chunk_res.get();
            let render_data =
                ChunkSolidRenderData::from_chunk(self.wgpu.clone(), &chunk, *chunk_coord);
            self.chunks.insert(*chunk_coord, render_data);
        }
    }
}

impl<K: PipelineKey> RenderPipeline<K> for SolidGeometryPipeline {
    fn label(&self) -> Option<&str> {
        Some("Solid Geometry Pipeline")
    }

    fn update(&mut self) -> Option<engine::graphics::pipeline::UpdateRequest> {
        /// TODO: Chunk updates will be done with a queue system to update only changed chunks.
        None
    }

    fn render(
        &self,
        controller: &engine::graphics::pipeline::controller::RenderController<K>,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let wgpu = controller.wgpu.get();
        let mut render_pass_desc = wgpu.render_pass(
            Some("Solid Geometry Pipeline Render Pass"),
            encoder,
            target,
            None,
            wgpu::LoadOp::Load,
        );

        for chunk_render_data in self.chunks.values() {
            chunk_render_data.draw(&mut render_pass_desc);
        }
    }
}

struct ChunkSolidRenderData {
    vertex_buffer: VertexBuffer<SolidBlockVertex>,
    index_buffer: IndexBuffer<u16>,
}

impl ChunkSolidRenderData {
    pub fn from_chunk(
        wgpu: ComponentHandle<WgpuRenderer>,
        chunk: &Chunk,
        chunk_coord: BlockPosition,
    ) -> Self {
        let (vertices, indices) = build_mesh_for_chunk(chunk);
        let wgpu = wgpu.get();
        let vertex_buffer = wgpu.vertex_buffer(
            &vertices,
            Some(&format!("Chunk Solid Vertex Buffer {:?}", chunk_coord)),
        );
        let index_buffer = wgpu.index_buffer(
            &indices,
            Some(&format!("Chunk Solid Index Buffer {:?}", chunk_coord)),
        );
        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    /// Draws the chunk's solid geometry.
    pub fn draw<'a>(&self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.buffer().slice(..));
        render_pass.set_index_buffer(
            self.index_buffer.buffer().slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..self.index_buffer.count() as u32, 0, 0..1);
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct SolidBlockVertex {
    pub position: Vec3,
    pub tex_coord: Vec2,
    pub texture_index: u32,
}

impl SolidBlockVertex {
    pub fn new(position: Vec3, tex_coord: Vec2, texture_index: u32) -> Self {
        Self {
            position,
            tex_coord,
            texture_index,
        }
    }
}

unsafe impl VertexLayout for SolidBlockVertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<SolidBlockVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x3, // position
            1 => Float32x2, // tex_coord
            2 => Uint32,    // texture_index
        ],
    };
}

fn build_mesh_for_chunk(chunk: &Chunk) -> (Vec<SolidBlockVertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                let block = chunk.data[x][y][z];
                if block.is_solid() {
                    mesh_block_at(
                        chunk,
                        bp(x as i64, y as i64, z as i64),
                        &mut vertices,
                        &mut indices,
                    );
                }
            }
        }
    }

    (vertices, indices)
}

fn mesh_block_at(
    chunk: &Chunk,
    chunk_pos: BlockPosition,
    vertices: &mut Vec<SolidBlockVertex>,
    indices: &mut Vec<u16>,
) {
    let mut push_face = |face: CardinalDirection| {
        let base_index = vertices.len() as u16;
        for (pos, uv) in FACE_TABLE[face as usize].iter() {
            let world_pos = Vec3::new(
                chunk_pos.0 as f32 + pos[0],
                chunk_pos.1 as f32 + pos[1],
                chunk_pos.2 as f32 + pos[2],
            );
            // FIXME: using the default no texture texture index 0 for now
            let vertex = SolidBlockVertex::new(world_pos, Vec2::new(uv[0], uv[1]), 0);
            vertices.push(vertex);
        }
        for &index in FACE_INDICES.iter() {
            indices.push(base_index + index);
        }
    };

    let chunk_rel = chunk_pos.chunk_normalize();

    for face in CardinalDirection::iter() {
        if !chunk.inspect_block(chunk_rel, face).is_solid() {
            push_face(face);
        }
    }
}
