use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use wgpu::VertexBufferLayout;

use crate::{
    ReadOnly, ReadOnlyString,
    graphics::lowlevel::{
        WgpuInstance,
        buf::{BufferLayout, Index16, ShaderType, WgpuBuffer},
    },
};

/// A 3D model consisting of vertex and index data.
///
/// Model has a maximum of 65536 vertices.
pub struct Model<'a> {
    data: ModelData,
    // Vertex buffer
    vbuf: WgpuBuffer<VertexData>,
    // 16-bit index buffer
    ibuf: WgpuBuffer<Index16>,
    wgpu: Rc<WgpuInstance<'a>>,
    label: ReadOnlyString,
}

impl<'a> Model<'a> {
    /// Creates a new Model from the given ModelData.
    pub fn new(wgpu: Rc<WgpuInstance<'a>>, label: &str, data: ModelData) -> Self {
        let vbuf_label = format!("{label} index buffer");
        let ibuf_label = format!("{label} vertex buffer");

        let ibuf = wgpu.create_buffer::<Index16>(
            wgpu::BufferUsages::INDEX,
            bytemuck::cast_slice::<_, Index16>(&data.indices),
            Some(&ibuf_label),
        );

        let vbuf = wgpu.create_buffer::<VertexData>(
            wgpu::BufferUsages::VERTEX,
            bytemuck::cast_slice::<_, VertexData>(&data.vertices),
            Some(&vbuf_label),
        );

        Self {
            wgpu,
            data,
            vbuf,
            ibuf,
            label: label.into(),
        }
    }

    /// Returns a reference to the model's vertex data.
    pub fn vertex_data(&self) -> &ReadOnly<VertexData> {
        &self.data.vertices
    }

    /// Returns a reference to the model's index data.
    pub fn index_data(&self) -> &ReadOnly<u16> {
        &self.data.indices
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelData {
    pub vertices: ReadOnly<VertexData>,
    pub indices: ReadOnly<u16>,
}

#[derive(Copy, Clone, Debug, Pod, Zeroable, PartialEq)]
#[repr(C)]
pub struct VertexData {
    /// Position in 3D model space. [0, 1]
    pub position: Vec3,
    /// Texture coordinates
    pub tex_coords: Vec2,
}

unsafe impl ShaderType for VertexData {
    fn layout() -> BufferLayout {
        BufferLayout::Vertex(VertexBufferLayout {
            array_stride: std::mem::size_of::<VertexData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<Vec3>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        })
    }
}
