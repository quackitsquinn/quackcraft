//! wgpu shader abstractions

use wgpu::VertexBufferLayout;

use crate::ReadOnlyString;

pub struct ShaderProgram<'a> {
    /// The shader module containing the shader code.
    pub module: wgpu::ShaderModule,
    /// The entry point for the vertex shader.
    pub vertex_entry_point: Option<ReadOnlyString>,
    /// The entry point for the fragment shader.
    pub fragment_entry_point: Option<ReadOnlyString>,
    /// The pipeline compilation options for this shader program.
    pub compilation_options: wgpu::PipelineCompilationOptions<'a>,
}

impl<'a> ShaderProgram<'a> {
    /// Creates a new ShaderProgram from the given parts.
    ///
    /// You probably want to use [`crate::graphics::WgpuInstance::load_shader`] to create the shader module.
    pub fn from_raw_parts(
        module: wgpu::ShaderModule,
        vertex_entry_point: Option<ReadOnlyString>,
        fragment_entry_point: Option<ReadOnlyString>,
        compilation_options: wgpu::PipelineCompilationOptions<'a>,
    ) -> Self {
        Self {
            module,
            vertex_entry_point,
            fragment_entry_point,
            compilation_options,
        }
    }

    /// Returns the vertex state for this shader program.
    pub fn vertex_state(&'a self, buffers: &'a [VertexBufferLayout]) -> wgpu::VertexState<'a> {
        wgpu::VertexState {
            module: &self.module,
            entry_point: self.vertex_entry_point.as_deref(),
            buffers,
            compilation_options: self.compilation_options.clone(),
        }
    }

    /// Returns the fragment state for this shader program.
    pub fn fragment_state(
        &'a self,
        targets: &'a [Option<wgpu::ColorTargetState>],
    ) -> Option<wgpu::FragmentState<'a>> {
        self.fragment_entry_point.as_ref()?;

        Some(wgpu::FragmentState {
            module: &self.module,
            entry_point: self.fragment_entry_point.as_deref(),
            targets,
            compilation_options: self.compilation_options.clone(),
        })
    }
}
