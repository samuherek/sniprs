mod gpt_query;

use std::io::{self, stdout, Write};

use crossterm::{
    cursor,
    execute,
    queue, 
    terminal::{self, ClearType},
    style::{self, Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    event::{self, KeyCode, KeyEvent, KeyModifiers, Event},
    ExecutableCommand
};
use clap::{Parser, Subcommand};
use std::io::Read;
use anyhow::{Result, Context};
use dirs;
use std::fs;
use gpt_query::query_gpt;

#[derive(Parser)]
#[command(name = "quickref")]
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


fn add_command() -> Result<String>{
    let history =  dirs::home_dir().expect("Can not find home dir").join(".zsh_history");
    println!("{:?}", history);

    // let mut file_content = Vec::new();
    // let mut file = File::open(&file_name).expect("Unable to open file");
    // file.read_to_end(&mut file_content).expect("Unable to read");
    // file_content

    let file = fs::read(&history).with_context(|| format!("Unable to read {:?}", &history))?;
    let content = String::from_utf8_lossy(&file);
    let command = content.lines().last().unwrap_or("");

    println!("{:?}", command);

    let res = query_gpt();
    println!("QUERY GPT:: {:?}", res);

    // let data = fs::read_to_string(&history).expect("can read to string");
    // let command = data.lines().last().unwrap_or("");
    // println!("{}", command);

    return Ok("".to_string());
}

fn list_command() -> anyhow::Result<()> {
    let lines = vec![
        "Line 1", "Line 2", "Line 3", "Line 4",
        "Line 5", "Line 6", "Line 7", "Line 8",
    ];

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

        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
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
                KeyCode::Char('q') | KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => {
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
            println!("None");
        }
    }
}
