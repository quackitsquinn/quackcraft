use engine::graphics::pipeline::controller::PipelineKey;

mod pipelines;

/// A collection of render pipelines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderPipelines {}

impl PipelineKey for RenderPipelines {}
