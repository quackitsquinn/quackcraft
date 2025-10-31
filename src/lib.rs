use log::info;

mod window;

/// The main game structure.
pub struct QuackCraft {
    window: window::GlfwWindow,
}

impl QuackCraft {
    /// Creates a new game instance.
    pub fn new() -> anyhow::Result<Self> {
        let window = window::GlfwWindow::new(800, 600, "Quackcraft")?;
        Ok(QuackCraft { window })
    }
}

pub fn run_game() -> anyhow::Result<()> {
    info!("Starting quackcraft");
    let mut qc = QuackCraft::new()?;
    while !qc.window.should_close() {
        qc.window.poll_events();
    }
    Ok(())
}
