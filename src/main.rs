use anyhow::bail;
use quackcraft::run_game;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    run_game()?;
    Ok(())
}
