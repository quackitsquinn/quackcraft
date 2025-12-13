use bytemuck::Pod;

/// A buffer for uniform data.
pub struct UniformBuffer<T>
where
    T: Pod,
{
    buffer: wgpu::Buffer,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Pod> UniformBuffer<T> {
    /// Creates a new UniformBuffer from a wgpu::Buffer.
    ///
    /// This function will panic if the buffer size is smaller than the size of type T.
    ///
    /// see also: [`crate::graphics::WgpuInstance::create_buffer`]
    /// # Safety
    /// The caller must ensure that the provided buffer is valid for the type T.
    pub unsafe fn from_raw_parts(buffer: wgpu::Buffer) -> Self {
        assert!(
            buffer.size() as usize >= std::mem::size_of::<T>(),
            "Buffer size is smaller than type T"
        );
        Self {
            buffer,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the underlying wgpu::Buffer.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
