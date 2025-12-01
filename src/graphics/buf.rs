use std::{marker::PhantomData, sync::Arc};

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
    Vertex(Arc<[wgpu::VertexAttribute]>),
    /// An index buffer format.
    Index(wgpu::IndexFormat),
}

/// A trait for types that can be used as buffer layouts.
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
