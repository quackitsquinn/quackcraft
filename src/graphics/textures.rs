use std::collections::HashMap;

use wgpu::{TextureFormat, TextureUsages};

use crate::{
    ReadOnly, ReadOnlyString,
    graphics::{Wgpu, image::Image, lowlevel::texture::Texture},
};

pub type TextureHandle = u32;

/// A structure managing a collection of textures.
/// This is effectively a texture atlas, packing multiple textures into a single GPU texture. The difference
/// is that this uses texture arrays instead of a single large texture, which greatly simplifies everything. The only limitation
/// is that all textures must have the same dimensions.
pub struct TextureCollection<'a> {
    textures: HashMap<String, TextureHandle>,
    buf: Vec<ReadOnly<u8>>,
    gpu_texture: Option<Texture<'a>>,
    label: Option<ReadOnlyString>,
    dimensions: (u32, u32),
    wgpu: Wgpu<'a>,
}

impl<'a> TextureCollection<'a> {
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

    /// Loads a texture from a file path.
    pub fn load_texture_from_file(
        &mut self,
        name: &str,
        path: &str,
    ) -> anyhow::Result<(TextureHandle, Image)> {
        let img = Image::from_file(path)?;
        let (width, height) = img.dimensions();
        if width != self.dimensions.0 || height != self.dimensions.1 {
            anyhow::bail!(
                "Texture dimensions do not match atlas dimensions: expected {}x{}, got {}x{}",
                self.dimensions.0,
                self.dimensions.1,
                width,
                height
            );
        }

        Ok((self.add_texture(name, img.pixel_bytes().clone()), img))
    }

    pub fn load_texture(
        &mut self,
        name: &str,
        data: &[u8],
    ) -> anyhow::Result<(TextureHandle, Image)> {
        let img = Image::from_mem(data)?;
        let (width, height) = img.dimensions();
        if width != self.dimensions.0 || height != self.dimensions.1 {
            anyhow::bail!(
                "Texture dimensions do not match atlas dimensions: expected {}x{}, got {}x{}",
                self.dimensions.0,
                self.dimensions.1,
                width,
                height
            );
        }

        Ok((self.add_texture(name, img.pixel_bytes().clone()), img))
    }

    /// Adds multiple textures from an iterator of (name, data) pairs.
    pub fn add_textures(
        &mut self,
        textures: impl IntoIterator<Item = (String, ReadOnly<u8>)>,
    ) -> TextureHandle {
        let base = self.buf.len() as TextureHandle;
        for (name, data) in textures {
            self.add_texture(&name, data);
        }
        base
    }

    pub fn push_invalid_texture(&mut self) -> TextureHandle {
        let mut data = vec![0u8; (self.dimensions.0 * self.dimensions.1 * 4) as usize];
        for (i, px) in data.chunks_mut(self.dimensions.0 as usize * 4).enumerate() {
            let is_bottom_half = i >= (self.dimensions.1 as usize / 2);
            for j in 0..(self.dimensions.0 as usize) {
                let offset = j * 4;
                let is_right_half = j >= (self.dimensions.0 as usize / 2);
                if !(is_bottom_half ^ is_right_half) {
                    px[offset + 3] = 255;
                    continue;
                }
                px[offset] = 255;
                px[offset + 1] = 0;
                px[offset + 2] = 255;
                px[offset + 3] = 255;
            }
        }

        self.add_texture("invalid_texture", data.into())
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
