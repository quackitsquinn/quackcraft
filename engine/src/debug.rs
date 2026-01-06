use std::{
    cell::RefCell,
    fmt::Debug,
    iter,
    rc::{Rc, Weak},
};

use log::info;
use wgpu_text::{
    BrushBuilder, TextBrush,
    glyph_brush::{Layout, Section, Text, ab_glyph::FontRef},
};

use crate::{
    ReadOnlyString,
    component::{ComponentHandle, ComponentStoreHandle},
    graphics::lowlevel::WgpuRenderer,
};

pub struct DebugRenderer {
    pub enabled: bool,
    brush: TextBrush<FontRef<'static>>,
    stats: Vec<Weak<DebugStatistic>>,
    wgpu: ComponentHandle<WgpuRenderer>,
}

impl Debug for DebugRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugRenderer")
            .field("enabled", &self.enabled)
            .field("stats_count", &self.stats.len())
            .finish()
    }
}

/// A type alias for a reference-counted debug statistic.
pub type DebugProvider = Rc<DebugStatistic>;

impl DebugRenderer {
    /// Creates a new debug renderer.
    pub fn new(state: &ComponentStoreHandle) -> anyhow::Result<DebugRenderer> {
        let wgpu = state.get::<WgpuRenderer>();
        let (render_width, render_height) = wgpu.dimensions();
        let render_format = wgpu.config.get().format;
        Ok(Self {
            brush: BrushBuilder::using_font_bytes(include_bytes!("../../FiraCode-Regular.ttf"))
                .expect("failed to create debug brush")
                .build(&wgpu.device, render_width, render_height, render_format),
            enabled: false,
            stats: Vec::new(),
            wgpu: state.handle_for::<WgpuRenderer>(),
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
        info!("Added debug statistic: {}", stat.label);
        stat
    }

    /// Renders the debug statistics on the screen.
    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        if !self.enabled {
            return;
        }

        let wgpu = self.wgpu.get();

        let mut pass = wgpu.render_pass(
            Some("Debug Renderer Pass"),
            encoder,
            view,
            None,
            wgpu::LoadOp::Load,
        );

        let mut section = Section {
            screen_position: (0.0, 0.0),
            bounds: (f32::INFINITY, f32::INFINITY),
            layout: Layout::default_wrap(),
            text: vec![],
        };

        // So i'll be the first to admit that this is a massive hack.
        // Im sure theres a better way to do this, but for now, this works.
        // We leak the boxes so that the Text objects can hold references to them.
        // Then after drawing, we reclaim the boxes and drop them.
        let mut leaked_boxes: Vec<&'static str> = Vec::new();

        for stat_weak in &self.stats {
            let stat = match stat_weak.upgrade() {
                Some(s) => s,
                None => continue,
            };

            let text = Box::leak(
                format!("{}: {}\n", stat.label, stat.value.borrow().as_str()).into_boxed_str(),
            );
            leaked_boxes.push(text);

            section
                .text
                .push(Text::new(text).with_color([1.0, 1.0, 1.0, 1.0]));
        }

        self.brush
            .queue(&wgpu.device, &wgpu.queue, iter::once(section))
            .expect("failed to queue debug text");

        self.brush.draw(&mut pass);

        for text in leaked_boxes {
            unsafe {
                let _ = Box::from_raw(text as *const str as *mut str);
            }
        }
    }

    /// Toggles the debug renderer on or off.
    pub fn toggle(&mut self) {
        info!(
            "Debug renderer {}",
            if self.enabled { "disabled" } else { "enabled" }
        );
        self.enabled = !self.enabled;
    }
}

/// A structure representing a debug statistic to be displayed.
pub struct DebugStatistic {
    pub label: ReadOnlyString,
    pub value: RefCell<String>,
}

impl Debug for DebugStatistic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugStatistic")
            .field("label", &self.label)
            .finish()
    }
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
