use std::ops::RangeBounds;

use bytemuck::{Pod, Zeroable};

pub struct IndexBuffer<T>
where
    T: IndexLayout,
{
    buffer: wgpu::Buffer,
    _marker: std::marker::PhantomData<T>,
}

impl<T: IndexLayout> IndexBuffer<T> {
    /// Creates a new IndexBuffer from a wgpu::Buffer.
    ///
    /// see also: [`crate::graphics::WgpuInstance::create_buffer`]
    /// # Safety
    /// The caller must ensure that the provided buffer is valid for the type T.
    pub unsafe fn from_raw_parts(buffer: wgpu::Buffer) -> Self {
        Self {
            buffer,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the underlying wgpu::Buffer.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Sets the index buffer on the given render pass for the specified range.
    pub fn set_on(&self, pass: &mut wgpu::RenderPass<'_>, range: impl RangeBounds<u64>) {
        pass.set_index_buffer(self.buffer.slice(range), T::FORMAT);
    }
}

trait Sealed {}

#[allow(private_bounds)]
pub trait IndexLayout: Pod + Zeroable + Sealed {
    const FORMAT: wgpu::IndexFormat;
}

impl Sealed for u16 {}
impl Sealed for u32 {}

impl IndexLayout for u16 {
    const FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;
}

impl IndexLayout for u32 {
    const FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32;
}
