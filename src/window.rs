use std::sync::Arc;

use glfw::{Glfw, GlfwReceiver, PWindow, Window};
use log::*;

use crate::{ReadOnlyString, graphics::callback::GlfwCallbackProxy};

#[derive(Debug)]
pub struct GlfwWindow {
    glfw: Glfw,
    /// The underlying GLFW window.
    pub window: Arc<PWindow>,
    pub event_receiver: GlfwReceiver<(f64, glfw::WindowEvent)>,
    pub mouse_pos_proxy: GlfwCallbackProxy<(f64, f64)>,
}

impl GlfwWindow {
    pub fn new(width: u32, height: u32, title: &str) -> anyhow::Result<Self> {
        let mut glfw = glfw::init(handle_glfw_error)
            .map_err(|e| anyhow::anyhow!("Failed to initialize GLFW: {}", e))?;

        let (mut window, event_receiver) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .ok_or_else(|| anyhow::anyhow!("Failed to create GLFW window"))?;

        let proxy = GlfwCallbackProxy::new();

        let closure_proxy = proxy.clone();
        window.set_cursor_pos_callback(move |_, x, y| {
            let proxy = closure_proxy.clone();
            proxy.invoke((x, y));
        });

        Ok(GlfwWindow {
            glfw,
            mouse_pos_proxy: proxy,
            window: Arc::new(window),
            event_receiver,
        })
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
    }

    pub fn register_mouse_pos_callback<F>(
        &self,
        label: Option<impl Into<ReadOnlyString>>,
        callback: F,
    ) where
        F: FnMut((f64, f64)) + 'static,
    {
        self.mouse_pos_proxy
            .add_target(callback, label.map(|l| l.into()));
    }
}

fn handle_glfw_error(error: glfw::Error, description: String) {
    error!("GLFW error {:?}: {}", error, description);
}
