pub mod term;

use crate::{Game, Point, Result, Status};
use rand::{thread_rng, Rng};
use std::time::{Duration, Instant};

pub enum Key {
    Tab,
    Left,
    Down,
    Up,
    Right,
    Char(char),
}

pub enum Colour {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Grey,
}

pub trait UIBackend {
    /// Initialise the user interface
    fn new() -> Result<Self>
    where
        Self: Sized;

    /// Draw to the display of the user interface
    fn draw(&mut self, pos: Point, str: &str, col: Colour) -> Result<()>;

    /// Flush display of the user interface
    fn flush(&mut self) -> Result<()>;

    /// Clear display of the user interface
    fn clear(&mut self) -> Result<()>;

    /// Receive input from the user interface
    fn input(&mut self, timeout: Duration) -> Result<Option<Key>>;

    /// Current size of the display of the user interface
    fn size(&self) -> Point;

    /// Print a message to the bottom of the display
    fn message(&mut self, msg: &str) -> Result<()>;

    /// De-initialise the user interface
    fn reset(&mut self);
}

pub struct UserInterface<T: UIBackend> {
    backend: T,
    centre: Option<Point>,
    guard: usize,
}

impl<T: UIBackend> UserInterface<T> {
    pub fn new(backend: T) -> Self {
        let centre = None;
        let guard = 0;

        UserInterface {
            backend,
            centre,
            guard,
        }
    }

    fn defender(&mut self, game: &mut Game, key: Key, set: &mut bool) -> isize {
        match key {
            Key::Tab => loop {
                self.guard = (self.guard + 1) % game.guards.len();
                if game.guards[self.guard].is_some() {
                    return 0;
                }
            },
            Key::Left => game.move_guard(self.guard, -1, 0),
            Key::Right => game.move_guard(self.guard, 1, 0),
            Key::Up => game.move_guard(self.guard, 0, -1),
            Key::Down => game.move_guard(self.guard, 0, 1),
            Key::Char(c) => match c {
                'q' => {
                    game.quit = Status::Quit;
                    return game.config.defender_actions;
                }
                '[' => game.rotate_guard(self.guard, false),
                ']' => game.rotate_guard(self.guard, true),
                '.' => (),
                ' ' => {
                    *set = true;
                }
                _ => return 0,
            },
        }
        1
    }

    fn player(&self, game: &mut Game, key: Key) -> isize {
        match key {
            Key::Tab => (),
            Key::Left => game.move_player(-1, 0),
            Key::Right => game.move_player(1, 0),
            Key::Up => game.move_player(0, -1),
            Key::Down => game.move_player(0, 1),
            Key::Char(c) => match c {
                'q' => {
                    game.quit = Status::Quit;
                    return game.config.attacker_actions;
                }
                '.' => (),
                _ => return 0,
            },
        }
        1
    }

    /// Display game status
    fn status(&mut self, game: &Game, ap: isize, rem: Duration) -> Result<()> {
        self.backend.message(&format!(
            "Your turn! Attackers: {}, Guards: {}, Actions: {}, Turn Time: {}s",
            game.positions.iter().filter(|&x| x.is_some()).count(),
            game.guards.iter().filter(|&x| x.is_some()).count(),
            ap,
            rem.as_secs(),
        ))?;
        Ok(())
    }

    /// Centre view of map on desired point
    fn map_to_display(&self, pos: Point) -> Option<Point> {
        let size = self.backend.size();
        let new = if let Some(centre) = self.centre {
            let dx = (size.0 / 2) as isize - centre.0 as isize;
            let dy = ((size.1 - 1) / 2) as isize - centre.1 as isize;
            let x = pos.0 as isize + dx;
            let y = pos.1 as isize + dy;
            if x >= 0 && y >= 0 {
                (x as u8, y as u8)
            } else {
                return None;
            }
        } else {
            (pos.0, pos.1)
        };

        if new.0 < size.0 && new.1 < size.1 - 1 {
            Some(new)
        } else {
            None
        }
    }

    pub fn message(&mut self, str: &str) -> Result<()> {
        self.backend.message(str)
    }

    /// Display current game state on terminal
    pub fn display(&mut self, game: &Game, defender: bool) -> Result<()> {
        self.backend.clear()?;

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
            if let Some(p) = self.map_to_display(pos) {
                self.backend.draw(p, &tile.to_string(), Colour::Grey)?;
            }
        }

        for i in 0..game.guards.len() {
            // View cones
            for (pos, tile) in game.view_cone(i).iter() {
                if let Some(p) = self.map_to_display(*pos) {
                    self.backend.draw(p, &tile.to_string(), Colour::Red)?;
                }
            }
        }

        for (i, guard) in game.guards.iter().enumerate() {
            if let Some((pos, _dir)) = guard {
                // Display guards
                if let Some(p) = self.map_to_display(*pos) {
                    let c = if defender && i == self.guard {
                        Colour::Yellow
                    } else {
                        Colour::Cyan
                    };
                    self.backend.draw(p, "G", c)?;
                }
            }
        }

        // Display players
        for (i, player) in game.positions.iter().enumerate() {
            if let Some(pos) = player {
                if !defender || game.visible(i) {
                    if let Some(p) = self.map_to_display(*pos) {
                        let c = if i != game.player {
                            Colour::White
                        } else {
                            Colour::Green
                        };
                        self.backend.draw(p, "A", c)?;
                    }
                }
            }
        }

        // Display player's target
        if !defender {
            if let Some(pos) = game.targets[game.player] {
                if let Some(p) = self.map_to_display(pos) {
                    self.backend.draw(p, "X", Colour::Green)?;
                }
            }
        }

        self.backend.flush()?;
        Ok(())
    }

    /// Event loop to get user input
    pub fn input(&mut self, game: &mut Game, defender: bool) -> Result<()> {
        let timer = Instant::now();

        let mut detected: isize = 3;
        let mut actions: isize = if defender {
            game.config.defender_actions
        } else {
            game.config.attacker_actions
        };
        while actions > 0 {
            if let Some(remaining) = game.config.turn_time.checked_sub(timer.elapsed()) {
                self.status(game, actions, remaining)?;
            } else {
                break;
            }

            if !defender && game.positions[game.player].is_none() {
                break;
            }

            if let Some(k) = self
                .backend
                .input(Duration::from_millis(game.config.input_timeout))?
            {
                actions -= if defender {
                    let mut x = true;
                    self.defender(game, k, &mut x)
                } else {
                    self.player(game, k)
                };
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

    /// Event loop to for placing guard positions
    pub fn place_guards(&mut self, game: &mut Game) -> Result<()> {
        let mut remaining: usize = game.config.num_guards;
        self.guard = 0;
        let mut final_choice = vec![];

        // Hide player positions
        let players = game.positions.clone();
        game.positions = vec![None; game.config.players];

        self.display(game, true)?;
        while remaining > 0 {
            self.message(&format!("{} guards remaining to place", remaining))?;

            if let Some(k) = self
                .backend
                .input(Duration::from_millis(game.config.input_timeout))?
            {
                let mut done = false;
                let _ = self.defender(game, k, &mut done);
                if done {
                    final_choice.push(game.guards[self.guard]);
                    game.guards[self.guard] = None;
                    remaining -= 1;
                    self.guard = game.guards.iter().position(|&x| x.is_some()).unwrap_or(0);
                }
            } else {
                continue;
            }

            self.display(game, true)?;
        }
        game.guards = final_choice;
        game.positions = players;

        Ok(())
    }

    /// Splash screen
    pub fn splash(&mut self) -> Result<()> {
        const SPLASH: &str = "██   ██  █████  ███    ██ ███████  ██████
██   ██ ██   ██ ████   ██    ███  ██    ██
███████ ███████ ██ ██  ██   ███   ██    ██
██   ██ ██   ██ ██  ██ ██  ███    ██    ██
██   ██ ██   ██ ██   ████ ███████  ██████

Version: 0.1.0
";

        self.backend.clear()?;
        let mut p = (5, 5);
        for line in SPLASH.lines() {
            self.backend.draw(p, line, Colour::Red)?;
            p.1 += 1;
        }
        self.backend.flush()?;
        Ok(())
    }

    /// De-initialise the user interface
    pub fn reset(&mut self) {
        self.backend.reset();
    }

    /// Idle screen
    pub fn idle(&mut self, begun: bool) -> Result<bool> {
        // Consume accidental input
        if let Some(Key::Char('q')) = self.backend.input(Duration::from_millis(100))? {
            return Ok(true);
        }
        if begun {
            // Draw @s at random points on the screen
            let mut rng = thread_rng();
            let size = self.backend.size();
            for _ in 0..(size.0 / 3) {
                let x = rng.gen_range(0..size.0);
                let y = rng.gen_range(0..(size.1 - 1));
                self.backend.draw((x, y), "@", Colour::Grey)?;
            }
        }
        self.message("Waiting for other players...")?;
        Ok(false)
    }
}

impl<T: UIBackend> Drop for UserInterface<T> {
    fn drop(&mut self) {
        self.reset();
    }
}
