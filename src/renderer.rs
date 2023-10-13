use std::io::Stdout;
use crossterm::{execute, terminal, style, cursor, queue};

pub struct Renderer<'a> {
    pub stdout: &'a mut Stdout,
}

impl<'a> Renderer<'a> {
    pub fn new(stdout: &'a mut Stdout) -> Self {
        return Renderer {
            stdout 
        };
    }

    pub fn enter_screen(&mut self) -> anyhow::Result<()> {
        execute!(self.stdout, terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;

        return Ok(());
    }

    pub fn clear_screen(&mut self) -> anyhow::Result<()> {
        queue!(
            self.stdout, 
            style::ResetColor, 
            terminal::Clear(terminal::ClearType::All), 
            cursor::Hide, 
            cursor::MoveTo(0,0)
            )?;

        return Ok(());
    }

    pub fn leave_screen(&mut self) -> anyhow::Result<()> {
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
