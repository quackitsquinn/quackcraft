use glfw::{Glfw, GlfwReceiver, PWindow};
use log::*;

#[derive(Debug)]
pub struct GlfwWindow {
    glfw: Glfw,
    window: PWindow,
    event_receiver: GlfwReceiver<(f64, glfw::WindowEvent)>,
}

impl GlfwWindow {
    pub fn new(width: u32, height: u32, title: &str) -> anyhow::Result<Self> {
        let mut glfw = glfw::init(handle_glfw_error)
            .map_err(|e| anyhow::anyhow!("Failed to initialize GLFW: {}", e))?;

        let (window, event_receiver) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .ok_or_else(|| anyhow::anyhow!("Failed to create GLFW window"))?;

        Ok(GlfwWindow {
            glfw,
            window,
            event_receiver,
        })
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
    }
}

fn handle_glfw_error(error: glfw::Error, description: String) {
    error!("GLFW error {:?}: {}", error, description);
}
