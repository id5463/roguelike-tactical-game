mod app;
mod command;
mod error;
mod map;
mod save;

use std::path::PathBuf;

use clap::Parser;

use crate::app::Game;

#[derive(Parser, Debug)]
#[command(name = "Save/Load the Lord", version, about = "ASCII roguelike auto-tactics game")]
pub struct CliArgs {
    #[arg(long, default_value = "0")]
    pub seed: u64,

    #[arg(long, default_value = "saves")]
    pub save_dir: String,

    #[arg(long)]
    pub headless: bool,
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    let save_dir = PathBuf::from(&args.save_dir);

    let seed = if args.seed == 0 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    } else {
        args.seed
    };

    let mut game = Game::new(100, 100, Some(seed), save_dir);
    game.run()?;

    Ok(())
}
