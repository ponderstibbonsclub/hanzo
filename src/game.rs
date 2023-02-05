use crate::{MsgToClient, MsgToServer, Result, ServerCli, UserInterface};
use rand::random;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Tile {
    Floor,
    Wall,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tile::Floor => write!(f, "·"),
            Tile::Wall => write!(f, "█"),
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
        // TODO map generation
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    pub address: String,
    pub players: usize,
    pub quit: bool,
    pub timer: Duration,
    pub defender: usize,
    pub pos: Option<Point>,
    pub positions: Vec<Option<Point>>,
    pub guards: Vec<Option<Point>>,
    pub map: Map,
}

impl Game {
    pub fn new(cli: ServerCli) -> Self {
        let map = Map::new(cli.len);
        let address = cli.address;
        let players = cli.players;
        let timer = Duration::from_secs((cli.timer * 60).into());

        let pos = None;
        let mut positions: Vec<Option<Point>> =
            (0..cli.players).map(|_| Some(map.random())).collect();
        let defender = random::<usize>() % cli.players;
        positions[defender] = None;
        let guards: Vec<Option<Point>> = (0..cli.guards).map(|_| Some(map.random())).collect();

        Game {
            address,
            players,
            quit: false,
            timer,
            defender,
            pos,
            positions,
            guards,
            map,
        }
    }

    pub fn turn(&self, player: usize, current: usize) -> MsgToClient {
        let turn = player == current;
        let defender = player == self.defender;
        let pos = self.positions[current];

        MsgToClient {
            turn,
            defender,
            pos,
            guards: self.guards.clone(),
            quit: self.quit,
        }
    }

    pub fn update(&mut self, msg: MsgToServer, current: usize) {
        self.positions[current] = msg.new;
        self.guards = msg.guards;
        self.quit = msg.quit;
    }

    pub fn play<T: UserInterface>(&mut self, defender: bool, ui: &mut T) -> Result<MsgToServer> {
        ui.input(self, defender)?;
        Ok(MsgToServer {
            new: self.pos,
            guards: self.guards.clone(),
            quit: self.quit,
        })
    }

    /// Is the player visible?
    pub fn visible(&self) -> bool {
        // TODO view-cones
        false
    }

    /// Move player position
    pub fn move_player(&mut self, dx: i16, dy: i16) {
        if let Some((x, y)) = self.pos {
            let x2 = (x as i16) + dx;
            let y2 = (y as i16) + dy;
            if let Some(tile) = self.map.at(x2 as usize, y2 as usize) {
                if tile == Tile::Floor {
                    self.pos = Some((x2 as u8, y2 as u8));
                }
            }
        }
    }
}
