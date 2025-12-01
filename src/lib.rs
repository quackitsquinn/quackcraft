use std::{cell::RefCell, iter, rc::Rc};

use glfw::WindowEvent;
use log::info;
use wgpu::Color;

use crate::graphics::WgpuInstance;

mod graphics;
mod window;

/// The main game structure.
pub struct QuackCraft<'a> {
    window: window::GlfwWindow,
    wgpu: Rc<RefCell<graphics::WgpuInstance<'a>>>,
}

impl<'a> QuackCraft<'a> {
    /// Creates a new game instance.
    pub fn new() -> anyhow::Result<Self> {
        let window = window::GlfwWindow::new(800, 600, "Quackcraft")?;
        let wgpu = smol::block_on(WgpuInstance::new(window.window.clone()))?;
        Ok(QuackCraft {
            window,
            wgpu: Rc::new(RefCell::new(wgpu)),
        })
    }

    fn rainbow(frame: u64) -> Color {
        let t = (frame as f64) * 0.02;
        Color {
            r: 0.5 + 0.5 * (t).sin(),
            g: 0.5 + 0.5 * (t + 2.0).sin(),
            b: 0.5 + 0.5 * (t + 4.0).sin(),
            a: 1.0,
        }
    }

    pub fn render(&mut self, frame: u64) -> anyhow::Result<()> {
        let wgpu = self.wgpu.borrow_mut();

        let mut encoder = wgpu.create_encoder(None);
        let (surface, view) = wgpu.current_view()?;

        wgpu.clear(Self::rainbow(frame), &mut encoder, &view);

        wgpu.submit_single(encoder.finish());
        surface.present();

        Ok(())
    }
}

pub fn run_game() -> anyhow::Result<()> {
    info!("Starting quackcraft");
    let mut qc = QuackCraft::new()?;
    let mut frame: u64 = 0;
    while !qc.window.should_close() {
        qc.window.poll_events();
        if let Some((_, event)) = qc.window.event_receiver.receive() {
            info!("Event: {:?}", event);
            match event {
                WindowEvent::Close => break,
                WindowEvent::Size(x, y) => {
                    qc.wgpu.borrow_mut().resize((x, y));
                }

                _ => {}
            }
        }
        qc.render(frame)?;
        frame += 1;
    }
    Ok(())
}
