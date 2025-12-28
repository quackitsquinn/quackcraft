use std::rc::Rc;

use wgpu::{CompareFunction, StoreOp, TextureFormat};

use crate::graphics::lowlevel::WgpuInstance;

pub struct DepthTexture<'a> {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    wgpu: Rc<WgpuInstance<'a>>,
}

impl<'a> DepthTexture<'a> {
    pub const TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;
    pub fn new(wgpu: Rc<WgpuInstance<'a>>) -> Self {
        let config = wgpu.config.get();
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = wgpu.device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = wgpu.comparing_sampler(CompareFunction::LessEqual);

        Self {
            texture,
            view,
            sampler,
            wgpu: wgpu.clone(),
        }
    }

    pub fn resize(&mut self) {
        let config = self.wgpu.config.get();
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        self.texture = self.wgpu.device.create_texture(&desc);
        self.view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
    }

    pub fn state(&self) -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: Self::TEXTURE_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }
    }

    pub fn attachment(&self) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: StoreOp::Store,
            }),
            stencil_ops: None,
        }
    }
}
