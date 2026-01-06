use engine::graphics::pipeline::controller::PipelineKey;

mod pipelines;

/// A collection of render pipelines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderPipelines {
    Clear,
}

impl PipelineKey for RenderPipelines {}
