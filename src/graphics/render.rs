use crate::{
    GameState,
    debug::{DebugProvider, DebugRenderer},
    graphics::{
        Wgpu,
        lowlevel::{WgpuInstance, depth::DepthTexture},
    },
    input::camera::CameraController,
    resource::Resource,
    window::GlfwWindow,
};

/// Module for rendering-related structures and functions.
pub struct RenderState {
    pub window: Resource<GlfwWindow>,
    // INFO: `window` must outlive `wgpu`!
    pub wgpu: Wgpu,
    pub depth_texture: DepthTexture,
    // TODO: CameraController should really own BindGroupLayout and BindGroup
    pub camera: (
        Resource<CameraController>,
        wgpu::BindGroupLayout,
        wgpu::BindGroup,
    ),
    pub debug_renderer: Resource<DebugRenderer>,
}

impl RenderState {
    /// Creates a new RenderState with the given window title and dimensions.
    pub async fn new(window_title: &str, window_dimensions: (u32, u32)) -> anyhow::Result<Self> {
        let window = GlfwWindow::new(window_dimensions.0, window_dimensions.1, window_title)
            .expect("Failed to create GLFW window");

        let wgpu = WgpuInstance::new(window.window.clone())
            .await
            .expect("Failed to initialize WGPU instance");

        let mut debug_renderer =
            DebugRenderer::new(wgpu.clone()).expect("Failed to create debug renderer");

        let camera = CameraController::create_main_camera(&wgpu, &window, &mut debug_renderer, 0);

        let depth_texture = DepthTexture::new(wgpu.clone());

        Ok(RenderState {
            window: window.into(),
            wgpu,
            depth_texture,
            camera,
            debug_renderer: debug_renderer.into(),
        })
    }
}

struct RenderStateDebugInformation {
    fps_stat: DebugProvider,
    frametime_stat: DebugProvider,
}

impl RenderStateDebugInformation {
    pub fn new(debug_renderer: &mut DebugRenderer) -> Self {
        let fps_stat = debug_renderer.add_statistic("fps", "0");
        let frametime_stat = debug_renderer.add_statistic("frametime (ms)", "0.0");
        Self {
            fps_stat,
            frametime_stat,
        }
    }

    pub fn update(&self, state: &GameState) {}
}
