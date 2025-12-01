//! wgpu shader abstractions

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
}
