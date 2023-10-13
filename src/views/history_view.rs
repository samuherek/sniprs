use std::io::Write;
use crossterm::{execute, queue, style, cursor};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use super::super::Renderer;
use super::super::history::get_history;

enum MoveDirection {
    Up,
    Down
}

type HistoryCommands = Vec<String>;

pub struct HistoryView {
    in_search_mode: bool,
    search_query: String,
    visible_commands: HistoryCommands, 
    selected_index: usize,
}

impl<'a> HistoryView {
    pub fn new() -> Self {
        return HistoryView {
            in_search_mode: false,
            search_query: String::from(""),
            visible_commands: Vec::new(),
            selected_index: 0,
        };
    }

    pub fn load_history(&mut self) -> anyhow::Result<()> {
        self.visible_commands = get_history(10)?;
        return Ok(());
    }

    pub fn is_empty(&self) -> bool {
        return self.visible_commands.len() == 0; 
    }

    pub fn get_selected(&self) -> String {
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

    pub fn render(&mut self, renderer: &mut Renderer<'a>) -> anyhow::Result<()>{
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
