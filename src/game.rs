use crate::{defaults, MsgToClient, MsgToServer, Result, ServerCli, UserInterface};
use rand::{
    distributions::{Distribution, Standard},
    random, Rng,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::time::Duration;

// View-cone parameters
const LENGTH: i16 = 16;
const WIDTH: usize = 10;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Running,
    AttackerVictory,
    DefenderVictory,
    Quit,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tile {
    Floor,
    Wall,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tile::Floor => write!(f, "."),
            Tile::Wall => write!(f, "#"),
        }
    }
}

impl From<char> for Tile {
    fn from(c: char) -> Tile {
        match c {
            '#' => Tile::Wall,
            _ => Tile::Floor,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Map {
    pub len: usize,
    pub buf: Vec<Tile>,
}

impl Map {
    pub fn new(len: usize) -> Self {
        // TODO better map generation
        let mut buf = vec![Tile::Floor; len * len];
        for tile in buf.iter_mut() {
            if random() {
                *tile = Tile::Wall;
            }
        }

        Map { len, buf }
    }

    pub fn at(&self, x: usize, y: usize) -> Option<Tile> {
        if x < self.len && y < self.len {
            Some(self.buf[y * self.len + x])
        } else {
            None
        }
    }

    pub fn at_ref(&mut self, x: usize, y: usize) -> Option<&Tile> {
        if x < self.len && y < self.len {
            Some(&self.buf[y * self.len + x])
        } else {
            None
        }
    }

    pub fn tiles(&self) -> Tiles {
        Tiles {
            index: 0,
            map: self,
        }
    }

    /// Find a random empty (floor) tile
    pub fn random(&self) -> Point {
        let mut x: u8 = random();
        let mut y: u8 = random();

        loop {
            if let Some(tile) = self.at(x as usize, y as usize) {
                if tile == Tile::Floor {
                    return (x, y);
                }
            }
            x = random();
            y = random();
        }
    }

    /// Shoot a line of sight
    pub fn bresenham(&self, start: (i16, i16), end: (i16, i16)) -> LineOfSight {
        let (mut x0, mut y0) = start;
        let (mut x1, mut y1) = end;
        let mut dx = x1 as f64 - x0 as f64;
        let mut dy = y1 as f64 - y0 as f64;

        // Rotate dimensions?
        let mut rotated = false;
        if dy.abs() > dx.abs() {
            (x0, y0) = (y0, x0);
            (x1, y1) = (y1, x1);
            rotated = true;
        }

        let mut sign = 1;
        if x0 > x1 {
            sign = -1;
            x0 *= -1;
            x1 *= -1;
        }
        let ystep = if y0 < y1 { 1 } else { -1 };

        dx = x1 as f64 - x0 as f64;
        dy = (y1 as f64 - y0 as f64).abs();
        let gradient = (dx, dy);
        let error = (dx / 2.0).floor();

        let current = (x0, y0);
        let end = x1;

        LineOfSight {
            current,
            end,
            gradient,
            error,
            sign,
            ystep,
            rotated,
            map: self,
        }
    }
}

impl From<&str> for Map {
    fn from(map: &str) -> Map {
        // quick and dirty, use at your peril
        let buf: Vec<Tile> = map
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.into())
            .collect();
        let len: usize = (buf.len() as f64).sqrt().floor() as usize;
        Map { len, buf }
    }
}

pub type Point = (u8, u8);

/// Iterator over Map's Tiles
pub struct Tiles<'a> {
    index: usize,
    map: &'a Map,
}

impl<'a> Iterator for Tiles<'a> {
    type Item = (Point, Tile);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.map.buf.len() {
            let x = self.index % self.map.len;
            let y = self.index / self.map.len;
            self.index += 1;
            Some(((x as u8, y as u8), self.map.at(x, y).unwrap()))
        } else {
            None
        }
    }
}

/// Line of sight across Map Tiles
pub struct LineOfSight<'a> {
    current: (i16, i16),
    end: i16,
    gradient: (f64, f64),
    error: f64,
    sign: i16,
    ystep: i16,
    rotated: bool,
    map: &'a Map,
}

impl<'a> Iterator for LineOfSight<'a> {
    type Item = (Point, Tile);

    fn next(&mut self) -> Option<Self::Item> {
        let new = if self.rotated {
            (self.current.1, self.sign * self.current.0)
        } else {
            (self.sign * self.current.0, self.current.1)
        };

        if self.current.0 > self.end {
            return None;
        }

        let (dx, dy) = self.gradient;
        self.current.0 += 1;
        self.error -= dy;
        if self.error < 0.0 {
            self.current.1 += self.ystep;
            self.error += dx;
        }

        if new.0 > 0 && new.1 > 0 {
            if let Some(tile) = self.map.at(new.0 as usize, new.1 as usize) {
                if tile == Tile::Floor {
                    return Some(((new.0 as u8, new.1 as u8), tile));
                }
            }
        }
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0..4) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    pub address: String,
    pub players: usize,
    pub quit: Status,
    pub timer: Duration,
    pub defender: usize,
    pub player: usize,
    pub positions: Vec<Option<Point>>,
    pub guards: Vec<Option<(Point, Direction)>>,
    pub targets: Vec<Option<Point>>,
    pub map: Map,
}

impl Game {
    pub fn new(cli: ServerCli) -> Self {
        let address = cli.address;
        let timer = Duration::from_secs((cli.timer * 60).into());
        let player = 0;

        let players: usize;
        let map: Map;
        let mut positions: Vec<Option<Point>>;
        let guards: Vec<Option<(Point, Direction)>>;
        let defender: usize;
        let mut targets: Vec<Option<Point>>;
        if cli.test.is_some() {
            players = defaults::PLAYERS;
            defender = defaults::DEFENDER;
            map = defaults::MAP.into();
            positions = defaults::POSITIONS.to_vec();
            guards = defaults::GUARDS.to_vec();
            targets = defaults::TARGETS.to_vec();
        } else {
            // TODO: better procedural generation
            players = cli.players;
            map = Map::new(cli.len);
            positions = (0..cli.players).map(|_| Some(map.random())).collect();
            defender = random::<usize>() % cli.players;
            positions[defender] = None;
            guards = (0..cli.guards)
                .map(|_| Some((map.random(), random::<Direction>())))
                .collect();
            targets = (0..cli.players).map(|_| Some(map.random())).collect();
            targets[defender] = None;
        }

        Game {
            address,
            players,
            quit: Status::Running,
            timer,
            defender,
            player,
            positions,
            guards,
            targets,
            map,
        }
    }

    /// Check for victory
    pub fn victory(&mut self) {
        // Attacker victory by objective
        let mut v = true;
        for i in 0..self.positions.len() {
            if self.positions[i].is_none() {
                continue;
            }

            if self.positions[i] != self.targets[i] {
                v = false;
            }
        }

        // Attacker victory by objective
        let u = self.guards.iter().filter(|&x| x.is_none()).count() == self.guards.len();

        if u || v {
            self.quit = Status::AttackerVictory;
            return;
        }

        // Defender victory by elimination
        if self.positions.iter().filter(|&x| x.is_none()).count() == self.guards.len() {
            self.quit = Status::DefenderVictory;
        }
    }

    /// Server-side turn processing
    pub fn turn(&self, player: usize, current: usize) -> MsgToClient {
        let turn = player == current;
        let defender = player == self.defender;
        let quit = if (defender && self.quit == Status::AttackerVictory)
            || (!defender && self.quit == Status::DefenderVictory)
        {
            Status::Quit
        } else {
            self.quit
        };

        MsgToClient {
            turn,
            defender,
            positions: self.positions.clone(),
            guards: self.guards.clone(),
            quit,
        }
    }

    /// Server-side turn processing
    pub fn update(&mut self, msg: MsgToServer, current: usize) {
        self.positions[current] = msg.new;
        self.guards = msg.guards;
        self.quit = msg.quit;
    }

    /// Client-side turn processing
    pub fn display<T: UserInterface>(&mut self, ui: &mut T, msg: &MsgToClient) -> Result<()> {
        self.positions = msg.positions.clone();
        self.guards = msg.guards.clone();
        self.quit = msg.quit;
        ui.display(self, msg.defender)?;
        Ok(())
    }

    /// Client-side turn processing
    pub fn play<T: UserInterface>(&mut self, defender: bool, ui: &mut T) -> Result<MsgToServer> {
        ui.input(self, defender)?;
        Ok(MsgToServer {
            new: self.positions[self.player],
            guards: self.guards.clone(),
            quit: self.quit,
        })
    }

    /// Tiles within guard's line-of-sight
    pub fn view_cone(&self, guard: usize) -> Vec<(Point, Tile)> {
        let mut cone = HashSet::new();
        let mut ends = HashSet::new();
        if let Some((pos, dir)) = self.guards[guard] {
            // Determine edge of cone
            for i in 0..WIDTH {
                match dir {
                    Direction::Up => {
                        ends.insert((pos.0 as i16 + i as i16, pos.1 as i16 - LENGTH));
                        ends.insert((pos.0 as i16 - i as i16, pos.1 as i16 - LENGTH));
                    }
                    Direction::Right => {
                        ends.insert((pos.0 as i16 + LENGTH, pos.1 as i16 + i as i16));
                        ends.insert((pos.0 as i16 + LENGTH, pos.1 as i16 - i as i16));
                    }
                    Direction::Down => {
                        ends.insert((pos.0 as i16 + i as i16, pos.1 as i16 + LENGTH));
                        ends.insert((pos.0 as i16 - i as i16, pos.1 as i16 + LENGTH));
                    }
                    Direction::Left => {
                        ends.insert((pos.0 as i16 - LENGTH, pos.1 as i16 + i as i16));
                        ends.insert((pos.0 as i16 - LENGTH, pos.1 as i16 - i as i16));
                    }
                }
            }

            // For each end-point shoot a line-of-sight
            for end in ends.iter() {
                for p in self.map.bresenham((pos.0 as i16, pos.1 as i16), *end) {
                    cone.insert(p);
                }
            }
        }
        Vec::from_iter(cone)
    }

    /// Is the player visible?
    pub fn visible(&self, player: usize) -> bool {
        if let Some(player) = self.positions[player] {
            for i in 0..self.guards.len() {
                for (view, _) in self.view_cone(i).iter() {
                    if *view == player {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Move player position
    pub fn move_player(&mut self, dx: i16, dy: i16) {
        if let Some((x, y)) = self.positions[self.player] {
            let x2 = (x as i16) + dx;
            let y2 = (y as i16) + dy;
            if let Some(tile) = self.map.at(x2 as usize, y2 as usize) {
                if tile == Tile::Floor {
                    self.positions[self.player] = Some((x2 as u8, y2 as u8));
                }
            }
        }
    }

    /// Move guard position
    pub fn move_guard(&mut self, guard: usize, dx: i16, dy: i16) {
        if let Some(((x, y), dir)) = self.guards[guard] {
            let x2 = (x as i16) + dx;
            let y2 = (y as i16) + dy;
            if let Some(tile) = self.map.at(x2 as usize, y2 as usize) {
                if tile == Tile::Floor {
                    self.guards[guard] = Some(((x2 as u8, y2 as u8), dir));
                }
            }
        }
    }

    /// Move guard direction
    pub fn rotate_guard(&mut self, guard: usize, clockwise: bool) {
        if let Some((pos, dir)) = self.guards[guard] {
            if clockwise {
                self.guards[guard] = match dir {
                    Direction::Up => Some((pos, Direction::Right)),
                    Direction::Right => Some((pos, Direction::Down)),
                    Direction::Down => Some((pos, Direction::Left)),
                    Direction::Left => Some((pos, Direction::Up)),
                }
            } else {
                self.guards[guard] = match dir {
                    Direction::Up => Some((pos, Direction::Left)),
                    Direction::Right => Some((pos, Direction::Up)),
                    Direction::Down => Some((pos, Direction::Right)),
                    Direction::Left => Some((pos, Direction::Down)),
                }
            }
        }
    }
}
