use std::{
    cell::RefCell,
    primitive,
    rc::{Rc, Weak},
    sync::Arc,
};

use anyhow::Context;
use wgpu::{
    self as w, Color, CommandBuffer, CommandEncoder, Device, DeviceDescriptor, Instance,
    InstanceDescriptor, Origin3d, PowerPreference, Queue, RenderPass, RequestAdapterOptions,
    StoreOp, Surface, SurfaceConfiguration, SurfaceTexture, TextureAspect, TextureView,
    naga::back::msl::sampler, util::DeviceExt,
};

use crate::graphics::{image::Image, shader::ShaderProgram, texture::Texture};

pub mod buf;
pub mod image;
pub mod shader;
pub mod texture;

pub struct WgpuInstance<'a> {
    pub instance: Instance,
    pub surface: Surface<'a>,
    pub device: Device,
    pub queue: Queue,
    pub config: RefCell<SurfaceConfiguration>,
    pub default_sampler: Option<wgpu::Sampler>,
    this: Weak<WgpuInstance<'a>>,
}

impl<'a> WgpuInstance<'a> {
    pub async fn new(window: Arc<glfw::PWindow>) -> anyhow::Result<Rc<Self>> {
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

        let this = Rc::new_cyclic(|weak| {
            let mut this = WgpuInstance {
                instance,
                surface,
                device,
                queue,
                config: RefCell::new(config),
                default_sampler: None,
                this: weak.clone(),
            };

            this.default_sampler =
                Some(this.sampler(Some("default sampler"), wgpu::AddressMode::ClampToEdge));

            this
        });

        Ok(this)
    }

    /// Resize the surface to the new size.
    ///
    /// # Panics
    /// Panics if the new size has a width or height less than or equal to zero.
    pub fn resize(&self, new_size: (i32, i32)) {
        debug_assert!(new_size.0 > 0 && new_size.1 > 0, "Window size <= 0");
        let mut cfg = self.config.borrow_mut();
        cfg.width = new_size.0 as u32;
        cfg.height = new_size.1 as u32;
        drop(cfg);
        self.surface.configure(&self.device, &self.config.borrow());
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

    /// Loads a shader module from WGSL source code.
    pub fn load_shader(
        &self,
        shader_source: &str,
        label: Option<&str>,
        vs_entry: Option<&str>,
        fs_entry: Option<&str>,
        compilation_options: wgpu::PipelineCompilationOptions<'a>,
    ) -> ShaderProgram<'a> {
        let module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label,
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

        ShaderProgram::from_raw_parts(
            module,
            vs_entry.map(Arc::from),
            fs_entry.map(Arc::from),
            compilation_options,
        )
    }

    /// Creates a texture with the given descriptor.
    pub fn create_texture(&self, desc: &wgpu::TextureDescriptor) -> wgpu::Texture {
        self.device.create_texture(desc)
    }

    /// Creates a texture from the given parameters, sized to the current surface configuration.
    ///
    /// This will upload the image pixel data to the texture.
    pub fn texture(
        &self,
        label: Option<&str>,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        image: &Image,
    ) -> Texture<'a> {
        let size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };
        // TODO: Mip level and sample count should probably be configurable.
        let text = self.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let text_layout = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                // TODO: Allow this to be configurable based on texture format.
                // Minecraft clone probably means that using a integer format is easier.
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        };

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &text,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            image.pixel_bytes(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width()),
                rows_per_image: Some(image.height()),
            },
            size,
        );

        let texture_view = text.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = self.default_sampler.clone().expect("no default sampler!");

        let sampler_layout = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };

        Texture::new(
            self.this.upgrade().clone().expect("WgpuInstance dropped!"),
            text,
            text_layout,
            sampler,
            sampler_layout,
            texture_view,
        )
    }

    /// Creates a bind group layout from the given descriptor.
    pub fn create_bind_group_layout(
        &self,
        desc: &wgpu::BindGroupLayoutDescriptor,
    ) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(desc)
    }

    /// Creates a bind group layout from the given entries.
    pub fn bind_group_layout(
        &self,
        label: Option<&str>,
        entries: &[wgpu::BindGroupLayoutEntry],
    ) -> wgpu::BindGroupLayout {
        self.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label, entries })
    }

    /// Creates a bind group from the given descriptor.
    pub fn create_bind_group(&self, desc: &wgpu::BindGroupDescriptor) -> wgpu::BindGroup {
        self.device.create_bind_group(desc)
    }

    /// Creates a bind group from the given parts.
    pub fn bind_group(
        &self,
        label: Option<&str>,
        layout: &wgpu::BindGroupLayout,
        entries: &[wgpu::BindGroupEntry],
    ) -> wgpu::BindGroup {
        self.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout,
            entries,
        })
    }

    /// Creates a sampler with the given descriptor.
    pub fn create_sampler(&self, desc: &wgpu::SamplerDescriptor) -> wgpu::Sampler {
        self.device.create_sampler(desc)
    }

    /// Creates a sampler with linear filtering and the specified address mode.
    pub fn sampler(&self, label: Option<&str>, address_mode: wgpu::AddressMode) -> wgpu::Sampler {
        self.create_sampler(&wgpu::SamplerDescriptor {
            label,
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            mag_filter: wgpu::FilterMode::Linear, // gotta love the n64
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }

    /// Creates a pipeline layout from the given descriptor.
    pub fn create_pipeline_layout(
        &self,
        desc: &wgpu::PipelineLayoutDescriptor,
    ) -> wgpu::PipelineLayout {
        self.device.create_pipeline_layout(desc)
    }

    /// Creates a pipeline layout from the given bind group layouts.
    pub fn pipeline_layout(
        &self,
        label: Option<&str>,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::PipelineLayout {
        self.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label,
            bind_group_layouts,
            push_constant_ranges: &[],
        })
    }

    /// Creates a render pipeline from the given descriptor.
    pub fn create_pipeline(&self, desc: &wgpu::RenderPipelineDescriptor) -> wgpu::RenderPipeline {
        self.device.create_render_pipeline(desc)
    }

    /// Creates a render pipeline from the given parts.
    pub fn pipeline(
        &'a self,
        label: Option<&str>,
        shader: &ShaderProgram,
        layout: &wgpu::PipelineLayout,
        buffers: &[wgpu::VertexBufferLayout<'a>],
        primitive: wgpu::PrimitiveState,
        targets: &[Option<wgpu::ColorTargetState>],
    ) -> wgpu::RenderPipeline {
        self.create_pipeline(&wgpu::RenderPipelineDescriptor {
            label,
            layout: Some(layout),
            vertex: shader.vertex_state(buffers),
            fragment: shader.fragment_state(targets.as_ref()),
            primitive,
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
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
    pub fn start_main_pass<'b>(
        &self,
        color: Color,
        encoder: &'b mut CommandEncoder,
        view: &TextureView,
    ) -> RenderPass<'b> {
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
        })
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
