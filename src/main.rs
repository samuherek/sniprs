mod gpt_query;

use std::io::{ stdout, Write};

use std::path::PathBuf;
use crossterm::{
    cursor,
    execute,
    queue, 
    terminal::{self, ClearType},
    style,
    event::{self, KeyCode, KeyEvent, KeyModifiers, Event},
    ExecutableCommand
};
use clap::{Parser, Subcommand};
use std::env;
use anyhow::anyhow;
use dirs;
use std::fs;
use gpt_query::query_gpt;

#[derive(Parser)]
#[command(name = "qvick")]
#[command(author = "Sam Uherek <samuherekbiz@gmail.com>")]
#[command(about = "Quick reference to commands", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add {},
    List
}

fn get_history_path() -> anyhow::Result<PathBuf> {
    let shell_path = env::var("SHELL")?;
    let shell_name = shell_path.rsplit('/').next().unwrap_or("");
    let home_dir = dirs::home_dir().unwrap_or(PathBuf::from("~/"));

    return match shell_name {
        "bash" => Ok(home_dir.join(".bash_history")),
        "zsh" => Ok(home_dir.join(".zsh_history")),
        _ => Err(anyhow!("We could not find your shell.")),
    }
}

/// Read the .zsh_history and get the last x lines from it.
fn get_history(line_count: usize) -> anyhow::Result<Vec<String>> {
    let history_path = get_history_path()?;
    let data: Vec<String> = String::from_utf8_lossy(&fs::read(&history_path)?)
        .lines()
        .rev()
        .take(line_count)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|l| l.to_string())
        .collect();

    return Ok(data);
}

fn list_command() -> anyhow::Result<()> {
    let lines = get_history(10)?;

    let mut selected_idx = 0;

    let mut stdout = stdout();

    execute!(stdout, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    loop {
        queue!(
            stdout, 
            style::ResetColor, 
            terminal::Clear(ClearType::All), 
            cursor::Hide, 
            cursor::MoveTo(0,0)
            )?;

        for (index, line) in lines.iter().enumerate() {
            let line = if index == selected_idx {
                format!("> {}", line)
            } else {
                format!("  {}", line)
            };

            queue!(stdout, style::Print(line))?;
            execute!(stdout, cursor::MoveToNextLine(1))?;
        }

        stdout.flush()?;

        if let Event::Key(KeyEvent { code,  .. }) = event::read()? {
            match code {
                KeyCode::Char('k') => {
                    selected_idx = selected_idx.saturating_sub(1);
                },
                KeyCode::Char('j') => {
                    selected_idx = (selected_idx + 1).min(lines.len() - 1);
                },
                KeyCode::Enter => {
                    execute!(
                        stdout, 
                        cursor::SetCursorStyle::DefaultUserShape
                        ).unwrap();
                    break;
                },
                KeyCode::Char('q') => {
                    execute!(
                        stdout, 
                        cursor::SetCursorStyle::DefaultUserShape
                        ).unwrap();
                    break;
                },
                _ => {}
            }
        }
    }

    execute!(
        stdout,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
        )?;

    terminal::disable_raw_mode()?;

    println!("{}", lines[selected_idx]);

    return Ok(());
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Add {}) => {
            println!("add stuff");
            // add_command().expect("Should exec the add command");
        },
        Some(Commands::List) => {
            list_command().unwrap(); 
        },
        None => {
            list_command().unwrap(); 
        }
    }
}
