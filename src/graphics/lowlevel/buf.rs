use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use bytemuck::{Pod, Zeroable};

pub struct WgpuBuffer<T>
where
    T: ShaderType,
{
    buffer: wgpu::Buffer,
    _marker: PhantomData<T>,
}

impl<T> WgpuBuffer<T>
where
    T: ShaderType,
{
    /// Creates a new WgpuBuffer from a wgpu::Buffer.
    ///
    /// see also: [`crate::graphics::WgpuInstance::create_buffer`]
    /// # Safety
    /// The caller must ensure that the provided buffer is valid for the type T.
    pub unsafe fn from_raw_parts(buffer: wgpu::Buffer) -> Self {
        Self {
            buffer,
            _marker: PhantomData,
        }
    }

    /// Returns the layout of the buffer.
    pub fn layout(&self) -> BufferLayout {
        T::layout()
    }

    /// Returns the underlying wgpu::Buffer.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

/// An enumeration of buffer attributes.
pub enum BufferLayout {
    /// A vertex buffer format.
    Vertex(wgpu::VertexBufferLayout<'static>),
    /// An index buffer format.
    Index(wgpu::IndexFormat),
    /// A uniform buffer format.
    Uniform,
}

impl BufferLayout {
    /// Returns true if the buffer layout is a vertex buffer.
    pub fn is_vertex(&self) -> bool {
        matches!(self, BufferLayout::Vertex(_))
    }

    /// Returns true if the buffer layout is an index buffer.
    pub fn is_index(&self) -> bool {
        matches!(self, BufferLayout::Index(_))
    }

    /// Returns true if the buffer layout is a uniform buffer.
    pub fn is_uniform(&self) -> bool {
        matches!(self, BufferLayout::Uniform)
    }

    /// Returns the vertex buffer layout if the buffer layout is a vertex buffer.
    pub fn as_vertex(&self) -> Option<wgpu::VertexBufferLayout<'static>> {
        match self {
            BufferLayout::Vertex(layout) => Some(layout.clone()),
            _ => None,
        }
    }

    /// Returns the index format if the buffer layout is an index buffer.
    pub fn as_index(&self) -> Option<wgpu::IndexFormat> {
        match self {
            BufferLayout::Index(format) => Some(*format),
            _ => None,
        }
    }
}

/// A trait for types that can be used as buffer layouts.
///
/// # Safety
/// The implementor must ensure that the given `BufferLayout` correctly describes the memory layout of the type.
pub unsafe trait ShaderType: Pod + Zeroable {
    /// Returns the vertex attributes for this buffer layout.
    ///
    /// The implementor must ensure that the returned attributes correctly describe the memory layout of the type.
    fn layout() -> BufferLayout;
}

/// A index buffer with 16-bit unsigned integer indices.
#[repr(transparent)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Index16(u16);

unsafe impl ShaderType for Index16 {
    fn layout() -> BufferLayout {
        BufferLayout::Index(wgpu::IndexFormat::Uint16)
    }
}

/// A index buffer with 32-bit unsigned integer indices.
#[repr(transparent)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Index32(u32);

unsafe impl ShaderType for Index32 {
    fn layout() -> BufferLayout {
        BufferLayout::Index(wgpu::IndexFormat::Uint32)
    }
}

/// A uniform buffer type.
#[repr(transparent)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Uniform<T>(T);

unsafe impl<T> ShaderType for Uniform<T>
where
    T: Pod + Zeroable,
{
    fn layout() -> BufferLayout {
        // Uniform buffers do not have a specific layout in wgpu.
        // The layout is determined by the shader and the data type T.
        // Therefore, we return a vertex buffer layout with no attributes.
        BufferLayout::Uniform
    }
}

impl<T> Deref for Uniform<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Uniform<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
