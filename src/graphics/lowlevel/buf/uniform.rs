use std::rc::Rc;

use bytemuck::Pod;

use crate::graphics::lowlevel::WgpuInstance;

/// A buffer for uniform data.
pub struct UniformBuffer<'a, T>
where
    T: Pod,
{
    buffer: wgpu::Buffer,
    wgpu: Rc<WgpuInstance<'a>>,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: Pod> UniformBuffer<'a, T> {
    /// Creates a new UniformBuffer from a wgpu::Buffer.
    ///
    /// This function will panic if the buffer size is smaller than the size of type T.
    ///
    /// see also: [`crate::graphics::WgpuInstance::create_buffer`]
    /// # Safety
    /// The caller must ensure that the provided buffer is valid for the type T.
    pub unsafe fn from_raw_parts(buffer: wgpu::Buffer, wgpu: Rc<WgpuInstance<'a>>) -> Self {
        assert!(
            buffer.size() as usize >= std::mem::size_of::<T>(),
            "Buffer size is smaller than type T"
        );
        Self {
            buffer,
            _marker: std::marker::PhantomData,
            wgpu,
        }
    }

    /// Returns the underlying wgpu::Buffer.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Writes data to the uniform buffer.
    pub fn write(&self, data: &T) {
        self.wgpu
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::bytes_of(data));
    }
}
