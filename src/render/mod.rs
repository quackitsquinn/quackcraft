use engine::graphics::pipeline::controller::PipelineKey;

pub mod block_textures;
pub mod pipelines;

/// A collection of render pipelines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderPipelines {
    Clear,
    SolidGeometry,
}

impl PipelineKey for RenderPipelines {}
