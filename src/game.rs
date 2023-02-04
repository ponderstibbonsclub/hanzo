use crate::{MsgToClient, MsgToServer, ServerCli};
use serde::{Deserialize, Serialize};
use std::thread::sleep;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Tile {
    Floor,
    Wall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Map {
    pub len: usize,
    pub buf: Vec<Tile>,
}

impl Map {
    pub fn new(len: usize) -> Self {
        // TODO
        let buf = vec![Tile::Floor; len];
        Map { len, buf }
    }

    pub fn at(&self, x: usize, y: usize) -> Tile {
        self.buf[y * self.len + x]
    }

    pub fn at_mut(&mut self, x: usize, y: usize) -> &Tile {
        &mut self.buf[y * self.len + x]
    }
}

pub type Point = (u8, u8);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    pub address: String,
    pub players: usize,
    pub quit: bool,
    defender: usize,
    pos: Option<Point>,
    guards: Vec<Option<Point>>,
    map: Map,
}

impl Game {
    pub fn new(cli: ServerCli) -> Self {
        let map = Map::new(cli.len);
        let address = cli.address;
        let players = cli.players;

        // TODO
        let pos = None;
        let guards = vec![None; cli.guards];
        let defender = 0;

        Game {
            address,
            players,
            quit: false,
            defender,
            pos,
            guards,
            map,
        }
    }

    pub fn turn(&self, player: usize, current: usize) -> MsgToClient {
        let turn = player == current;
        let defender = player == self.defender;
        // TODO
        MsgToClient {
            turn,
            defender,
            pos: self.pos,
            guards: self.guards.clone(),
            quit: self.quit,
        }
    }

    pub fn update(&self, msg: MsgToServer) {
        // TODO
    }

    pub fn display(&self, msg: &MsgToClient) {
        // TODO
    }

    pub fn player(&self) -> MsgToServer {
        // TODO
        sleep(Duration::from_millis(400));
        MsgToServer {
            new: self.pos,
            guards: self.guards.clone(),
            quit: self.quit,
        }
    }

    pub fn defender(&self) -> MsgToServer {
        // TODO
        sleep(Duration::from_millis(400));
        MsgToServer {
            new: self.pos,
            guards: self.guards.clone(),
            quit: self.quit,
        }
    }
}
