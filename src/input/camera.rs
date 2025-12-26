use std::{cell::RefCell, rc::Rc};

use glam::{Mat4, Vec2, Vec3};

use crate::graphics::{Wgpu, camera::Camera, lowlevel::buf::UniformBuffer};

pub struct CameraController<'a> {
    pub pos: Vec2,
    /// Pitch and yaw rotation.
    pub rot: Vec2,
    camera: Camera,
    uniform: UniformBuffer<'a, Mat4>,
    wgpu: Wgpu<'a>,
}

impl CameraController<'_> {
    pub fn new<'a>(wgpu: Wgpu<'a>) -> CameraController<'a> {
        let camera = Camera::new(
            wgpu.config.borrow().width as f32 / wgpu.config.borrow().height as f32,
            0.1,
            100.0, // TODO: render distance setting? i think this is in world units
        );

        let uniform = wgpu.uniform_buffer(&camera.projection_view_matrix(), Some("Camera Uniform"));
        CameraController {
            wgpu,
            camera,
            uniform,
            pos: Vec2::ZERO,
            rot: Vec2::ZERO,
        }
    }

    pub fn process_rot(&mut self, direction: Vec2) {
        let sensitivity = 0.1;
        self.rot.x += direction.x * sensitivity;
        self.rot.y += direction.y * sensitivity;

        // Clamp the pitch to avoid flipping
        self.rot.y = self.rot.y.clamp(-89.0, 89.0);

        let yaw_radians = self.rot.x.to_radians();
        let pitch_radians = self.rot.y.to_radians();

        self.camera.set_orientation(yaw_radians, pitch_radians);
    }

    /// Returns a clone of the camera's uniform buffer.
    pub fn uniform(&self) -> UniformBuffer<'_, Mat4> {
        self.uniform.clone()
    }

    /// Creates a bind group layout for the camera uniform buffer.
    pub fn bind_group_layout(&self, binding: u32) -> wgpu::BindGroupLayout {
        self.wgpu.bind_group_layout(
            Some("camera bind group layout"),
            &[wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        )
    }

    /// Writes the current camera matrix to the uniform buffer.
    pub fn flush(&mut self) {
        let matrix = self.camera.projection_view_matrix();
        self.uniform.write(&matrix);
    }

    /// Sets the camera to look at a specific target point.
    pub fn look_at(&mut self, target: Vec3) {
        self.camera.look_at(target);
        self.flush();
    }

    /// Creates a bind group for the camera uniform buffer.
    pub fn bind_group_with_layout(
        &self,
        layout: &wgpu::BindGroupLayout,
        binding: u32,
    ) -> wgpu::BindGroup {
        self.wgpu.bind_group(
            Some("camera bind group"),
            layout,
            &[wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::Buffer(
                    self.uniform.buffer().as_entire_buffer_binding(),
                ),
            }],
        )
    }

    /// Creates a bind group for the camera uniform buffer.
    pub fn bind_group(&self, binding: u32) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let layout = self.bind_group_layout(binding);
        (
            layout.clone(),
            self.bind_group_with_layout(&layout, binding),
        )
    }
}
