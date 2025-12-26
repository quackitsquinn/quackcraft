use std::collections::HashMap;

use wgpu::{TextureFormat, TextureUsages};

use crate::{
    ReadOnly, ReadOnlyString,
    graphics::{Wgpu, lowlevel::texture::Texture},
};

pub type TextureHandle = u32;
pub struct Textures<'a> {
    textures: HashMap<String, TextureHandle>,
    buf: Vec<ReadOnly<u8>>,
    gpu_texture: Option<Texture<'a>>,
    label: Option<ReadOnlyString>,
    dimensions: (u32, u32),
    wgpu: Wgpu<'a>,
}

impl<'a> Textures<'a> {
    pub fn new(
        wgpu: Wgpu<'a>,
        label: Option<impl Into<ReadOnlyString>>,
        dimensions: (u32, u32),
    ) -> Self {
        Self {
            textures: HashMap::new(),
            buf: Vec::new(),
            gpu_texture: None,
            label: label.map(|l| l.into()),
            wgpu,
            dimensions,
        }
    }

    /// Adds a new texture from raw RGBA8 data.
    pub fn add_texture(&mut self, name: &str, data: ReadOnly<u8>) -> TextureHandle {
        let handle = self.buf.len() as TextureHandle;
        self.buf.push(data);
        self.textures.insert(name.to_string(), handle);
        handle
    }

    /// Retrieves a texture handle by name.
    pub fn get_texture(&self, name: &str) -> Option<&TextureHandle> {
        self.textures.get(name)
    }

    /// Returns the GPU texture, creating it if it doesn't exist.
    pub fn gpu_texture(&mut self) -> Texture<'a> {
        if self.gpu_texture.is_some() {
            return self.gpu_texture.as_ref().unwrap().clone();
        }

        let texture = self.wgpu.texture(
            self.label.as_deref(),
            TextureFormat::Rgba8Unorm,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            self.dimensions,
            &self.buf,
        );

        self.gpu_texture = Some(texture);
        self.gpu_texture.as_ref().unwrap().clone()
    }
}
