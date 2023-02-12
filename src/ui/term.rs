use crate::{Game, Point, Result, Status, UserInterface};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{PrintStyledContent, Stylize},
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Stdout, Write};
use std::ops::Drop;
use std::time::{Duration, Instant};

const TIMEOUT: u64 = 300;
const ATTACKER_ACTIONS: isize = 5;
const DEFENDER_ACTIONS: isize = 8;

pub struct Terminal {
    stdout: Stdout,
    size: (u16, u16),
    centre: Option<Point>,
    guard: usize,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        let size = terminal::size()?;
        let centre = None;
        execute!(stdout, Hide)?;
        let guard = 0;

        Ok(Terminal {
            stdout,
            size,
            centre,
            guard,
        })
    }

    fn defender(&mut self, game: &mut Game, key: KeyCode) -> isize {
        match key {
            KeyCode::Tab => loop {
                self.guard = (self.guard + 1) % game.guards.len();
                if game.guards[self.guard].is_some() {
                    return 0;
                }
            },
            KeyCode::Left => game.move_guard(self.guard, -1, 0),
            KeyCode::Right => game.move_guard(self.guard, 1, 0),
            KeyCode::Up => game.move_guard(self.guard, 0, -1),
            KeyCode::Down => game.move_guard(self.guard, 0, 1),
            KeyCode::Char(c) => match c {
                'q' => {
                    game.quit = Status::Quit;
                    return DEFENDER_ACTIONS;
                }
                '[' => game.rotate_guard(self.guard, false),
                ']' => game.rotate_guard(self.guard, true),
                '.' => (),
                _ => return 0,
            },
            _ => return 0,
        }
        1
    }

    fn player(&self, game: &mut Game, key: KeyCode) -> isize {
        match key {
            KeyCode::Left => game.move_player(-1, 0),
            KeyCode::Right => game.move_player(1, 0),
            KeyCode::Up => game.move_player(0, -1),
            KeyCode::Down => game.move_player(0, 1),
            KeyCode::Char(c) => match c {
                'q' => {
                    game.quit = Status::Quit;
                    return ATTACKER_ACTIONS;
                }
                '.' => (),
                _ => return 0,
            },
            _ => return 0,
        }
        1
    }

    /// Display game status
    fn status(&mut self, game: &Game, ap: isize, rem: Duration) -> Result<()> {
        self.message(&format!(
            "Your turn! Attackers: {}, Guards: {}, Actions: {}, Turn Time: {}s",
            game.positions.iter().filter(|&x| x.is_some()).count(),
            game.guards.iter().filter(|&x| x.is_some()).count(),
            ap,
            rem.as_secs(),
        ))?;
        Ok(())
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

        if defender {
            self.centre = if let Some((pos, _)) = game.guards[self.guard] {
                Some(pos)
            } else {
                None
            };
        } else {
            self.centre = game.positions[game.player];
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

        for (i, guard) in game.guards.iter().enumerate() {
            if let Some((pos, _dir)) = guard {
                // View cones
                for (pos, tile) in game.view_cone(i).iter() {
                    if let Some((x, y)) = self.map_to_display(*pos) {
                        queue!(
                            self.stdout,
                            MoveTo(x, y),
                            PrintStyledContent(tile.to_string().on_red())
                        )?;
                    }
                }

                // Display guards
                if let Some((x, y)) = self.map_to_display(*pos) {
                    let g = if defender && i == self.guard {
                        "G".yellow()
                    } else {
                        "G".cyan()
                    };
                    queue!(self.stdout, MoveTo(x, y), PrintStyledContent(g))?;
                }
            }
        }

        // Display players
        for (i, player) in game.positions.iter().enumerate() {
            if let Some(pos) = player {
                if !defender || game.visible(i) {
                    if let Some((x, y)) = self.map_to_display(*pos) {
                        let p = if i != game.player {
                            "A".white()
                        } else {
                            "A".green().bold()
                        };
                        queue!(self.stdout, MoveTo(x, y), PrintStyledContent(p))?;
                    }
                }
            }
        }

        // Display player's target
        if !defender {
            if let Some(pos) = game.targets[game.player] {
                if let Some((x, y)) = self.map_to_display(pos) {
                    queue!(self.stdout, MoveTo(x, y), PrintStyledContent("X".green()))?;
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
            PrintStyledContent(msg.magenta()),
            Clear(ClearType::UntilNewLine),
        )?;
        self.stdout.flush()?;
        Ok(())
    }

    /// Event loop to get user input from terminal
    fn input(&mut self, game: &mut Game, defender: bool) -> Result<()> {
        let timer = Instant::now();

        let mut detected: isize = 3;
        let mut actions: isize = if defender {
            DEFENDER_ACTIONS
        } else {
            ATTACKER_ACTIONS
        };
        while actions > 0 {
            if let Some(remaining) = game.timer.checked_sub(timer.elapsed()) {
                self.status(game, actions, remaining)?;
            } else {
                break;
            }

            if !defender && game.positions[game.player].is_none() {
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

            // Check for guard elimination
            for guard in game.guards.iter_mut() {
                if let Some(pos) = game.positions[game.player] {
                    if let Some((guard_pos, _)) = guard {
                        if *guard_pos == pos {
                            *guard = None;
                        }
                    }
                }
            }

            // Check for attacker elimination (on their turn only)
            if game.visible(game.player) {
                detected -= 1;
            }
            if detected == 0 {
                game.positions[game.player] = None;
            }

            self.display(game, defender)?;
            if !defender && game.positions[game.player] == game.targets[game.player] {
                game.positions[game.player] = None;
                break;
            }
        }

        Ok(())
    }

    /// Reset the terminal back to original state
    fn reset(&mut self) {
        execute!(self.stdout, MoveTo(0, self.size.1 - 1), Show).ok();
        terminal::disable_raw_mode().ok();
    }
}
