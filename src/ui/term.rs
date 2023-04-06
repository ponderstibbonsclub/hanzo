use crate::{Colour, Key, Point, Result, UIBackend};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{Print, SetForegroundColor, SetBackgroundColor, ResetColor, Color, PrintStyledContent, Stylize},
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Stdout, Write};
use std::time::Duration;

pub struct Terminal {
    stdout: Stdout,
    size: (u16, u16),
}

// Map from Hanzo's colours to Crossterm's
fn colour(col: Colour) -> Color {
    match col {
        Colour::Black => Color::Black,
        Colour::Red => Color::Red,
        Colour::Green => Color::Green,
        Colour::Yellow => Color::Yellow,
        Colour::Blue => Color::Blue,
        Colour::Magenta => Color::Magenta,
        Colour::Cyan => Color::Cyan,
        Colour::White => Color::White,
        Colour::Grey => Color::Grey,
        Colour::Reset => Color::Reset,
    }
}

impl UIBackend for Terminal {
    /// Initialise the terminal for user interface
    fn new() -> Result<Self> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        let size = terminal::size()?;
        execute!(stdout, Hide)?;

        Ok(Terminal { stdout, size })
    }

    /// Display current game state on terminal
    fn draw(&mut self, pos: Point, str: &str, fg: Colour, bg: Colour) -> Result<()> {
        queue!(
            self.stdout,
            MoveTo(pos.0 as u16, pos.1 as u16),
            SetForegroundColor(colour(fg)),
            SetBackgroundColor(colour(bg)),
            Print(str),
            ResetColor
        )?;
        Ok(())
    }

    /// Flush writes to terminal
    fn flush(&mut self) -> Result<()> {
        self.stdout.flush()?;
        Ok(())
    }

    /// Clear terminal
    fn clear(&mut self) -> Result<()> {
        queue!(self.stdout, Clear(ClearType::All))?;
        Ok(())
    }

    /// Receive user input from terminal
    fn input(&mut self, timeout: Duration) -> Result<Option<Key>> {
        if poll(timeout)? {
            match read()? {
                Event::Key(event) => {
                    // Map from Crossterm keys to our basic set
                    let k = match event.code {
                        KeyCode::Tab => Key::Tab,
                        KeyCode::Left => Key::Left,
                        KeyCode::Down => Key::Down,
                        KeyCode::Up => Key::Up,
                        KeyCode::Right => Key::Right,
                        KeyCode::Char(c) => Key::Char(c),
                        _ => return Ok(None),
                    };
                    return Ok(Some(k));
                }
                Event::Resize(w, h) => {
                    self.size = (w, h);
                }
                _ => (),
            }
        }
        Ok(None)
    }

    /// Return current size of terminal
    fn size(&self) -> Point {
        let max = u8::MAX;
        let x = if self.size.0 > max as u16 {
            max
        } else {
            self.size.0 as u8
        };
        let y = if self.size.1 > max as u16 {
            max
        } else {
            self.size.1 as u8
        };
        (x, y)
    }

    /// Write a message to the bottom of the terminal
    fn message(&mut self, msg: &str) -> Result<()> {
        queue!(
            self.stdout,
            MoveTo(0, self.size.1 - 1),
            PrintStyledContent(msg.magenta()),
            Clear(ClearType::UntilNewLine),
        )?;
        self.flush()?;
        Ok(())
    }

    /// Reset the terminal back to original state
    fn reset(&mut self) {
        execute!(self.stdout, MoveTo(0, self.size.1 - 1), Show).ok();
        terminal::disable_raw_mode().ok();
        println!();
    }
}
