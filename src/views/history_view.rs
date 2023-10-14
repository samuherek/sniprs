use std::io::Write;
use crossterm::{execute, queue, style, cursor};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crate::gpt_query;

use super::super::Renderer;
use super::super::history::get_history;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use super::super::config;

enum MoveDirection {
    Up,
    Down
}

enum ViewState {
    List,
    Search,
    ApiLoading,
    Saved,
}

type HistoryCommands = Vec<String>;

pub struct HistoryView {
    view: ViewState,
    search_query: String,
    visible_commands: HistoryCommands, 
    all_commands: HistoryCommands,
    visible_limit: usize,
    selected_index: usize,
}

impl<'a> HistoryView {
    pub fn new() -> Self {
        return HistoryView {
            view: ViewState::List,
            search_query: String::from(""),
            visible_commands: Vec::new(),
            all_commands: Vec::new(),
            visible_limit: 10,
            selected_index: 0,
        };
    }

    pub fn load_history(&mut self) -> anyhow::Result<()> {
        self.all_commands = get_history()?;
        self.assign_query_commands()?;
        return Ok(());
    }

    fn assign_query_commands(&mut self) -> anyhow::Result<()>{
        let matcher = SkimMatcherV2::default();

        if self.search_query.trim().is_empty() {
            self.visible_commands = self.all_commands
                .iter()
                .take(self.visible_limit)
                .cloned()
                .collect();

        } else {
            self.visible_commands = self.all_commands.iter()
                .filter_map(|item| {
                    return matcher.fuzzy_match(item, &self.search_query).map(|score| (item, score));
                })
                //.filter(|&(_, score)| score > 20)  // Use a score threshold to filter results.
                .map(|(item, _)| item.clone())
                .take(self.visible_limit)
                .collect();
        }

        return Ok(());
    }

    pub fn is_empty(&self) -> bool {
        return self.visible_commands.len() == 0; 
    }

    pub fn get_selected(&self) -> String {
        return self.visible_commands[self.selected_index].clone();
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

    fn save_command(&mut self) -> anyhow::Result<()> {
        let command = self.get_selected();

        config::init_dotfolder()?;
        config::save_command(&command)?;

        gpt_query::query_gpt(&command)?;

        self.view = ViewState::Saved;

        return Ok(());
    }

    pub fn render(&mut self, renderer: &mut Renderer<'a>) -> anyhow::Result<()>{
        
        loop {
            renderer.clear_screen()?;
            render_view(&self.view, renderer)?;

            match self.view {
                ViewState::List => {
                    render_list(&self, renderer)?;

                    if let Event::Key(KeyEvent { code,  .. }) = event::read()? {
                        match code {
                            KeyCode::Char('k') => {
                                self.move_selected_index(MoveDirection::Up);
                            },
                            KeyCode::Char('j') => {
                                self.move_selected_index(MoveDirection::Down);
                            },
                            KeyCode::Char('/') => {
                                self.selected_index = 0;
                                self.view = ViewState::Search;
                                self.search_query = String::from("");
                            }
                            KeyCode::Enter => {
                                self.view = ViewState::ApiLoading;
                                self.save_command()?;
                            },
                            KeyCode::Char('q') => {
                                break;
                            },
                            _ => {}
                        }
                    }
                },
                ViewState::Search => {
                    render_list(&self, renderer)?;
                    render_query(&self, renderer)?;

                    if let Event::Key(KeyEvent {code, ..}) = event::read()? {
                        match code {
                            KeyCode::Esc => {
                                self.view = ViewState::List;
                                self.search_query = String::from("");
                            }
                            KeyCode::Backspace => {
                                self.search_query.pop();
                                self.assign_query_commands()?;
                            },
                            KeyCode::Char(c) => {
                                self.search_query.push(c);
                                self.assign_query_commands()?;
                            },
                            KeyCode::Enter => {
                                self.view = ViewState::List;
                            }
                            _ => {}
                        }
                    }
                },
                ViewState::ApiLoading => {
                    if let Event::Key(KeyEvent {code, ..}) = event::read()? {
                        match code {
                            KeyCode::Char('q') => {
                                break;
                            },
                            _ => {}
                        }
                    }

                    render_api_loading(&self, renderer)?;
                },
                ViewState::Saved => {
                    if let Event::Key(KeyEvent {code, ..}) = event::read()? {
                        match code {
                            KeyCode::Char('q') => {
                                break;
                            },
                            _ => {}
                        }
                    }
                    render_done(&self, renderer)?;
                }
            }

            render_view(&self.view, renderer)?;

            renderer.stdout.flush()?;
        }

        return Ok(());
    }
}

fn render_done<'a>(view: &HistoryView, renderer: &mut Renderer<'a>) -> anyhow::Result<()> {
   execute!(
        renderer.stdout,
        cursor::MoveTo(0,0),
        style::Print(format!(
                "Saved {} to {}", 
                view.search_query.clone(),
                view.get_selected(),
                )
            )
       )?; 

   return Ok(());
}

fn render_api_loading<'a>(view: &HistoryView, renderer: &mut Renderer<'a>) -> anyhow::Result<()> {
   execute!(
        renderer.stdout,
        cursor::MoveTo(0,0),
        style::Print(format!("Fetching chatGPT description for: {}", view.search_query.clone()))
       )?; 

   return Ok(());
}

fn render_list<'a>(view: &HistoryView, renderer: &mut Renderer<'a>) -> anyhow::Result<()> {
    for (index, line) in view.visible_commands.iter().enumerate() {
        let line = if index == view.selected_index {
            format!("> {}", line)
        } else {
            format!("  {}", line)
        };

        queue!(renderer.stdout, style::Print(line))?;
        execute!(renderer.stdout, cursor::MoveToNextLine(1))?;
    }

    return Ok(());
}

fn render_view<'a>(view: &ViewState, renderer: &mut Renderer<'a>) -> anyhow::Result<()> {
    let (_, rows) = crossterm::terminal::size()?;

    execute!(
        renderer.stdout,
        cursor::MoveTo(0, 0),
        )?;

    queue!(
        renderer.stdout, 
        style::Print(format!("{}", match view {
            ViewState::List => "List" ,
            ViewState::Search => "Search" ,
            ViewState::ApiLoading => "Api loading" ,
            ViewState::Saved => "Saved" ,
            }))
        )?;


    execute!(
        renderer.stdout, 
        cursor::MoveTo(0,1)
        )?;

    //renderer.stdout.flush()?;

    return Ok(());        
}

fn render_query<'a>(view: &HistoryView, renderer: &mut Renderer<'a>) -> anyhow::Result<()> {
    let (_, rows) = crossterm::terminal::size()?;

    execute!(
        renderer.stdout,
        cursor::MoveTo(0, rows - 1),
        cursor::SavePosition
        )?;

    queue!(
        renderer.stdout, 
        style::Print(format!("/{}", view.search_query.clone()))
        )?;

    execute!(
        renderer.stdout, 
        cursor::RestorePosition
        )?;

    //renderer.stdout.flush()?;

    return Ok(());        
}
