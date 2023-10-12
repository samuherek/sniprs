mod gpt_query;

use std::io::{ stdout, Stdout, Write};

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
#[command(name = "comrs")]
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

const CONFIG_DIR: &str = ".comrs";

fn get_config_path() -> PathBuf {
    return dirs::home_dir().expect("Could not determine home dir").join(CONFIG_DIR);
}



fn get_history_path() -> anyhow::Result<PathBuf> {
    let shell_path = env::var("SHELL")?;
    let shell_name = shell_path.rsplit('/').next().unwrap_or("");
    let home_dir = dirs::home_dir().expect("Could not determine home dir");

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


enum MoveDirection {
    Up,
    Down
}

type HistoryCommands = Vec<String>;

struct HistoryView {
    in_search_mode: bool,
    search_query: String,
    visible_commands: HistoryCommands, 
    selected_index: usize,
}

impl<'a> HistoryView {
    fn new() -> Self {
        return HistoryView {
            in_search_mode: false,
            search_query: String::from(""),
            visible_commands: Vec::new(),
            selected_index: 0,
        };
    }

    fn load_history(&mut self) -> anyhow::Result<()> {
        self.visible_commands = get_history(10)?;
        return Ok(());
    }

    fn is_empty(&self) -> bool {
        return self.visible_commands.len() == 0; 
    }

    fn get_selected(&self) -> String {
        return self.visible_commands[self.selected_index].clone();
    } 

    fn render_list(&self, renderer: &mut Renderer<'a>) -> anyhow::Result<()> {
        for (index, line) in self.visible_commands.iter().enumerate() {
            let line = if index == self.selected_index {
                format!("> {}", line)
            } else {
                format!("  {}", line)
            };

            queue!(renderer.stdout, style::Print(line))?;
            execute!(renderer.stdout, cursor::MoveToNextLine(1))?;
        }

        return Ok(());
    }

    fn render_query(&self, renderer: &mut Renderer<'a>) -> anyhow::Result<()> {
        let (_, rows) = crossterm::terminal::size()?;

        execute!(
            renderer.stdout,
            cursor::MoveTo(0, rows - 1),
            cursor::SavePosition
            )?;

        queue!(
            renderer.stdout, 
            style::Print(format!("/{}", self.search_query.clone()))
            )?;

        execute!(
            renderer.stdout, 
            cursor::RestorePosition
            )?;

        renderer.stdout.flush()?;

        return Ok(());        
    }

    fn move_selected_index(&mut self, direction: MoveDirection) {
        if self.visible_commands.len() == 0 {
            return;
        }

        match direction{
            MoveDirection::Up => {
                self.selected_index = self.selected_index.saturating_sub(1);
            },
            MoveDirection::Down => {
                self.selected_index = (self.selected_index + 1).min(self.visible_commands.len() - 1);
            }
        }
    }

    fn render(&mut self, renderer: &mut Renderer<'a>) -> anyhow::Result<()>{
        loop {
            renderer.clear_screen()?;

            self.render_list(renderer)?;

            if self.in_search_mode {
                self.render_query(renderer)?;
            }

            renderer.stdout.flush()?;


            if self.in_search_mode {
                if let Event::Key(KeyEvent {code, ..}) = event::read()? {
                    match code {
                        KeyCode::Esc => {
                            self.in_search_mode = false;
                            self.search_query = String::from("");
                        }
                        KeyCode::Backspace => {
                            self.search_query.pop();
                        },
                        KeyCode::Char(c) => {
                            self.search_query.push(c);
                        }
                        _ => {}
                    }
                }
            } else {
            if let Event::Key(KeyEvent { code,  .. }) = event::read()? {
                match code {
                    KeyCode::Char('k') => {
                        self.move_selected_index(MoveDirection::Up);
                    },
                    KeyCode::Char('j') => {
                        self.move_selected_index(MoveDirection::Down);
                    },
                    KeyCode::Char('/') => {
                        self.in_search_mode = true;
                    }
                    KeyCode::Enter => {
                        //execute!(
                         //   stdout, 
                         //   cursor::SetCursorStyle::DefaultUserShape
                         //   ).unwrap();
                        break;
                    },
                    KeyCode::Char('q') => {
                        // execute!(
                         //   stdout, 
                         //   cursor::SetCursorStyle::DefaultUserShape
                         //   ).unwrap();
                        break;
                    },
                    _ => {}
                }
            }
            }
        }

        return Ok(());
    }
}

struct Renderer<'a> {
    stdout: &'a mut Stdout,
}

impl<'a> Renderer<'a> {
    fn new(stdout: &'a mut Stdout) -> Self {
        return Renderer {
            stdout 
        };
    }

    fn enter_screen(&mut self) -> anyhow::Result<()> {
        execute!(self.stdout, terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;

        return Ok(());
    }

    fn clear_screen(&mut self) -> anyhow::Result<()> {
        queue!(
            self.stdout, 
            style::ResetColor, 
            terminal::Clear(ClearType::All), 
            cursor::Hide, 
            cursor::MoveTo(0,0)
            )?;

        return Ok(());
    }

    fn leave_screen(&mut self) -> anyhow::Result<()> {
        execute!(
            self.stdout,
            style::ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen
            )?;

        terminal::disable_raw_mode()?;

        return Ok(());
    }
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



fn main() -> anyhow::Result<()>{
    let cli = Cli::parse();

    let mut stdout = stdout();
    let mut renderer = Renderer::new(&mut stdout);


    match &cli.command {
        Some(Commands::Add {}) => {
            println!("add stuff");
            // add_command().expect("Should exec the add command");
        },
        Some(Commands::List) => {
            list_command().unwrap(); 
        },
        None => {
            let mut history = HistoryView::new();
            history.load_history()?;

            if history.is_empty() {
                println!("Your command history is empty.");
                return Ok(());
            }

            renderer.enter_screen()?;
            history.render(&mut renderer)?;
            renderer.leave_screen()?;

            println!("{}", history.get_selected());

           // list_command().unwrap(); 
        }
    }

    return Ok(());
}
