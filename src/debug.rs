use std::{
    cell::RefCell,
    iter,
    rc::{Rc, Weak},
};

use log::error;
use wgpu_text::{
    BrushBuilder, TextBrush,
    glyph_brush::{Layout, Section, Text, ab_glyph::FontRef},
};

use crate::{ReadOnlyString, graphics::Wgpu};

pub struct DebugRenderer<'a> {
    pub enabled: bool,
    brush: TextBrush<FontRef<'static>>,
    stats: Vec<Weak<DebugStatistic>>,
    wgpu: Wgpu<'a>,
}

/// A type alias for a reference-counted debug statistic.
pub type DebugProvider = Rc<DebugStatistic>;

impl<'a> DebugRenderer<'a> {
    /// Creates a new debug renderer.
    pub fn new(wgpu: Wgpu<'a>) -> anyhow::Result<Self> {
        let (render_width, render_height) = wgpu.dimensions();
        let render_format = wgpu.config.borrow().format;
        Ok(Self {
            brush: BrushBuilder::using_font_bytes(include_bytes!("../FiraCode-Regular.ttf"))
                .expect("failed to create debug brush")
                .build(&wgpu.device, render_width, render_height, render_format),
            enabled: false,
            stats: Vec::new(),
            wgpu,
        })
    }

    /// Adds a new debug statistic to be displayed.
    pub fn add_statistic(
        &mut self,
        label: impl Into<ReadOnlyString>,
        initial_value: impl Into<String>,
    ) -> Rc<DebugStatistic> {
        let stat = Rc::new(DebugStatistic::new(label, initial_value));
        self.stats.push(Rc::downgrade(&stat.clone()));
        stat
    }

    /// Renders the debug statistics on the screen.
    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        if !self.enabled {
            return;
        }

        let mut pass = self.wgpu.render_pass(
            Some("Debug Renderer Pass"),
            encoder,
            view,
            None,
            wgpu::LoadOp::Load,
        );

        let mut text_strings = Vec::new();
        let mut vertical_offset = 0;
        for stat_weak in &self.stats {
            let stat = match stat_weak.upgrade() {
                Some(s) => s,
                None => continue,
            };

            let text = format!("{}: {}", stat.label, stat.value.borrow().as_str());
            text_strings.push(text);
            let text_ref = text_strings.last().unwrap();

            let _ = self
                .brush
                .queue(
                    &self.wgpu.device,
                    &self.wgpu.queue,
                    iter::once(Section {
                        screen_position: (0.0, vertical_offset as f32),
                        bounds: (f32::INFINITY, f32::INFINITY),
                        layout: Layout::default_single_line(),
                        text: vec![
                            Text::new(text_ref)
                                .with_color([1.0, 1.0, 1.0, 1.0])
                                .with_scale(16.0),
                        ],
                    }),
                )
                .inspect_err(|f| error!("Failed to draw debug line: {}", f));

            vertical_offset += 18;
        }

        self.brush.draw(&mut pass);
    }

    /// Toggles the debug renderer on or off.
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

/// A structure representing a debug statistic to be displayed.
pub struct DebugStatistic {
    pub label: ReadOnlyString,
    pub value: RefCell<String>,
}

impl DebugStatistic {
    /// Creates a new debug statistic with the given label and initial value.
    pub fn new(label: impl Into<ReadOnlyString>, initial_value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: RefCell::new(initial_value.into()),
        }
    }

    /// Updates the value of the debug statistic.
    pub fn update_value(&self, new_value: impl ToString) {
        *self.value.borrow_mut() = new_value.to_string();
    }
}
