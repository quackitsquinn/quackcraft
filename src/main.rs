use anyhow::bail;
use quackcraft::run_game;

mod window;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    run_game()?;
    Ok(())
}
