mod gpt_query;
mod renderer;
mod history;
mod cli;
mod views;
mod config;

use crate::renderer::Renderer;
use crate::views::history_view::HistoryView;
use crate::cli::{Cli, Commands};
use std::path::PathBuf;
use clap::Parser;


const CONFIG_DIR: &str = ".sniprs";

fn get_config_path() -> PathBuf {
    return dirs::home_dir().expect("Could not determine home dir").join(CONFIG_DIR);
}


fn main() -> anyhow::Result<()>{
    let cli = Cli::parse();

    let mut stdout = std::io::stdout();
    let mut renderer = Renderer::new(&mut stdout);

    match &cli.command {
        Some(Commands::Add) => {
            println!("add stuff");
        },
        Some(Commands::List) => {
            println!("List commands");
        },
        None => {
            let mut history = HistoryView::new();
            history.load_history()?;

            if history.is_empty() {
                println!("Your command history is empty.");
                return Ok(());
            }

            // It renders the alternative screen to keep the old terminal view
            renderer.enter_screen()?;
            // Interactive render that will be in a loop until it's exited
            history.render(&mut renderer)?;
            // Cleanup and restoring the original termianl view
            renderer.leave_screen()?;

        }
    }

    return Ok(());
}
