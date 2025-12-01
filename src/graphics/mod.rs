use std::sync::Arc;

use anyhow::Context;
use wgpu::{
    self as w, Color, CommandBuffer, CommandEncoder, Device, DeviceDescriptor, Instance,
    InstanceDescriptor, PowerPreference, Queue, RenderPass, RequestAdapterOptions, StoreOp,
    Surface, SurfaceConfiguration, SurfaceTexture, TextureView, util::DeviceExt,
};

pub mod buf;

pub struct WgpuInstance<'a> {
    pub instance: Instance,
    pub surface: Surface<'a>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
}

impl<'a> WgpuInstance<'a> {
    pub async fn new(window: Arc<glfw::PWindow>) -> anyhow::Result<Self> {
        let size = window.get_size();

        let instance = Instance::new(&InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| anyhow::anyhow!("Failed to create surface: {:?}", e))?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .with_context(|| "Failed to find an appropriate adapter")?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await
            .with_context(|| "Failed to create device")?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0 as u32,
            height: size.1 as u32,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self {
            instance,
            surface,
            device,
            queue,
            config,
        })
    }

    /// Resize the surface to the new size.
    ///
    /// # Panics
    /// Panics if the new size has a width or height less than or equal to zero.
    pub fn resize(&mut self, new_size: (i32, i32)) {
        debug_assert!(new_size.0 > 0 && new_size.1 > 0, "Window size <= 0");
        self.config.width = new_size.0 as u32;
        self.config.height = new_size.1 as u32;
        self.surface.configure(&self.device, &self.config);
    }

    /// Creates a command encoder.
    pub fn create_encoder(&self, label: Option<&str>) -> CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label })
    }

    /// Creates a buffer with the given usage and data.
    pub fn create_buffer<T>(
        &self,
        usage: wgpu::BufferUsages,
        data: &[T],
        label: Option<&str>,
    ) -> buf::WgpuBuffer<T>
    where
        T: buf::ShaderType,
    {
        let buffer = self
            .device
            .create_buffer_init(&w::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(data),
                usage,
            });

        // Safety: The buffer is valid for type T as it was created from a slice of T.
        unsafe { buf::WgpuBuffer::from_raw_parts(buffer) }
    }

    /// Acquires the current texture view from the surface.
    pub fn current_view(&self) -> anyhow::Result<(SurfaceTexture, TextureView)> {
        let frame = self
            .surface
            .get_current_texture()
            .with_context(|| "Failed to acquire next swap chain texture")?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        Ok((frame, view))
    }

    /// Clears the given texture view with the specified color using the provided command encoder.
    pub fn clear(&self, color: Color, encoder: &mut CommandEncoder, view: &TextureView) {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("clear render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
    }

    /// Submits a single command encoder to the queue. This is a direct wrapper around `Queue::submit`.
    pub fn submit_single(&self, encoder: CommandBuffer) {
        self.queue.submit(std::iter::once(encoder));
    }

    /// Submits multiple command buffers to the queue.
    pub fn submit<I: IntoIterator<Item = CommandBuffer>>(&self, bufs: I) {
        self.queue.submit(bufs);
    }
}
