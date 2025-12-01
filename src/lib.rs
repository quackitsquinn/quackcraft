use std::{cell::RefCell, iter, rc::Rc, sync::Arc};

use glfw::WindowEvent;
use log::info;
use wgpu::Color;

use crate::graphics::WgpuInstance;

/// A read-only string type.
pub type ReadOnlyString = Arc<str>;
/// A read-only slice type.
pub type ReadOnly<T> = Arc<[T]>;

mod graphics;
mod window;

/// The main game structure.
pub struct QuackCraft<'a> {
    window: window::GlfwWindow,
    wgpu: Rc<RefCell<graphics::WgpuInstance<'a>>>,
    pipelines: Vec<wgpu::RenderPipeline>,
}

impl<'a> QuackCraft<'a> {
    /// Creates a new game instance.
    pub fn new() -> anyhow::Result<Self> {
        let window = window::GlfwWindow::new(800, 600, "Quackcraft")?;
        let wgpu = smol::block_on(WgpuInstance::new(window.window.clone()))?;

        let program = wgpu.load_shader(
            include_str!("../shaders/test.wgsl"),
            Some("test_shader"),
            Some("vs_main"),
            Some("fs_main"),
            wgpu::PipelineCompilationOptions::default(),
        );

        let layout = wgpu.pipeline_layout(None, &[]);

        let pipeline = wgpu.pipeline(
            Some("main pipeline"),
            &program,
            &layout,
            &[],
            wgpu::PrimitiveState::default(),
            &[Some(wgpu::ColorTargetState {
                format: wgpu.config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        );

        Ok(QuackCraft {
            window,
            wgpu: Rc::new(RefCell::new(wgpu)),
            pipelines: vec![pipeline],
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

        let mut pass = wgpu.start_main_pass(Self::rainbow(frame), &mut encoder, &view);

        pass.set_pipeline(&self.pipelines[0]);
        pass.draw(0..3, 0..1);

        drop(pass);

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
