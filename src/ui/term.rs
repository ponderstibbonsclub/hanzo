use crate::{Game, Point, Result, UserInterface};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{Print, PrintStyledContent, Stylize},
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Stdout, Write};
use std::ops::Drop;
use std::time::{Duration, Instant};

const TIMEOUT: u64 = 100;
const ACTIONS: isize = 3;

pub struct Terminal {
    stdout: Stdout,
    size: (u16, u16),
    centre: Option<Point>,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        let size = terminal::size()?;
        let centre = None;
        execute!(stdout, Hide)?;

        Ok(Terminal {
            stdout,
            size,
            centre,
        })
    }

    fn defender(&self, game: &mut Game, key: KeyCode) -> isize {
        // TODO control guards
        match key {
            KeyCode::Char(c) => match c {
                'q' => {
                    game.quit = true;
                    return ACTIONS;
                }
                '.' => (),
                _ => return 0,
            },
            _ => return 0,
        }
        1
    }

    fn player(&self, game: &mut Game, key: KeyCode) -> isize {
        // TODO control player
        match key {
            KeyCode::Left => game.move_player(-1, 0),
            KeyCode::Right => game.move_player(1, 0),
            KeyCode::Up => game.move_player(0, -1),
            KeyCode::Down => game.move_player(0, 1),
            KeyCode::Char(c) => match c {
                'q' => {
                    game.quit = true;
                    return ACTIONS;
                }
                '.' => (),
                _ => return 0,
            },
            _ => return 0,
        }
        1
    }

    /// Centre view of map on desired point
    fn map_to_display(&self, pos: Point) -> Option<(u16, u16)> {
        let new = if let Some(centre) = self.centre {
            let dx = (self.size.0 / 2) as isize - centre.0 as isize;
            let dy = ((self.size.1 - 1) / 2) as isize - centre.1 as isize;
            let x = pos.0 as isize + dx;
            let y = pos.1 as isize + dy;
            if x >= 0 && y >= 0 {
                (x as u16, y as u16)
            } else {
                return None;
            }
        } else {
            (pos.0 as u16, pos.1 as u16)
        };

        if new.0 < self.size.0 && new.1 < self.size.1 - 1 {
            Some(new)
        } else {
            None
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.reset();
    }
}

impl UserInterface for Terminal {
    /// Display current game state on terminal
    fn display(&mut self, game: &Game, defender: bool) -> Result<()> {
        queue!(self.stdout, Clear(ClearType::All))?;
        self.message("Waiting for other player(s)...")?;

        if !defender {
            self.centre = game.pos;
        }

        // Display map
        for (pos, tile) in game.map.tiles() {
            if let Some((x, y)) = self.map_to_display(pos) {
                queue!(
                    self.stdout,
                    MoveTo(x, y),
                    PrintStyledContent(tile.to_string().grey()),
                )?;
            }
        }

        // Display guards
        for pos in game.guards.iter().flatten() {
            if let Some((x, y)) = self.map_to_display(*pos) {
                queue!(self.stdout, MoveTo(x, y), PrintStyledContent("☻".cyan()))?;
            }
            // TODO view-cones
        }

        // Display player
        if !defender || game.visible() {
            if let Some(pos) = game.pos {
                if let Some((x, y)) = self.map_to_display(pos) {
                    queue!(self.stdout, MoveTo(x, y), PrintStyledContent("☺".magenta()))?;
                }
            }
        }

        self.stdout.flush()?;
        Ok(())
    }

    /// Write a message to the bottom of the terminal
    fn message(&mut self, msg: &str) -> Result<()> {
        queue!(
            self.stdout,
            MoveTo(0, self.size.1 - 1),
            Print(msg),
            Clear(ClearType::UntilNewLine),
        )?;
        self.stdout.flush()?;
        Ok(())
    }

    /// Event loop to get user input from terminal
    fn input(&mut self, game: &mut Game, defender: bool) -> Result<()> {
        let timer = Instant::now();

        let mut actions: isize = ACTIONS;
        while actions > 0 {
            if let Some(remaining) = game.timer.checked_sub(timer.elapsed()) {
                self.message(&format!("{} seconds remaining...", remaining.as_secs()))?;
            } else {
                break;
            }

            if poll(Duration::from_millis(TIMEOUT))? {
                match read()? {
                    Event::Key(event) => {
                        actions -= if defender {
                            self.defender(game, event.code)
                        } else {
                            self.player(game, event.code)
                        };
                    }
                    Event::Resize(w, h) => {
                        self.size = (w, h);
                    }
                    _ => (),
                }
            } else {
                continue;
            }

            self.display(game, defender)?;
        }

        Ok(())
    }

    /// Reset the terminal back to original state
    fn reset(&mut self) {
        execute!(self.stdout, MoveTo(0, self.size.1 - 1), Show).ok();
        terminal::disable_raw_mode().ok();
    }
}
