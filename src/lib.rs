use clap::Parser;
use log::info;
use serde::{Deserialize, Serialize};
use std::thread::sleep;
use std::time::Duration;

pub const GRID_SIZE: usize = 32;
pub const NUM_GUARDS: usize = 8;

#[derive(Parser)]
pub struct ServerCli {
    /// IP address of server
    pub address: String,
    /// Number of players
    pub players: usize,
}

#[derive(Parser)]
pub struct ClientCli {
    /// IP address of server
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Tile {
    Floor,
    Wall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Map(pub Vec<Tile>);

impl Map {
    pub fn at(&self, x: usize, y: usize) -> Tile {
        self.0[y * GRID_SIZE + x]
    }

    pub fn at_mut(&mut self, x: usize, y: usize) -> &Tile {
        &mut self.0[y * GRID_SIZE + x]
    }
}

impl Default for Map {
    fn default() -> Self {
        Map(vec![Tile::Floor; GRID_SIZE * GRID_SIZE])
    }
}

type Point = (u8, u8);

#[derive(Serialize, Deserialize, Debug)]
// Information needed by client each turn
pub struct ToClient {
    // Is it the player's turn?
    pub turn: bool,
    // Player's position
    pub pos: Option<Point>,
    // Guards' positions
    pub guards: [Option<Point>; NUM_GUARDS],
}

#[derive(Serialize, Deserialize, Debug)]
// Information needed by server each turn
pub struct ToServer {
    // New position of player's character
    pub new: Option<Point>,
}

pub struct Player {
    pub map: Map, // TODO
}

impl Player {
    pub fn display(&mut self, state: &ToClient) {
        // Display latest update
    }

    pub fn turn(&mut self, state: &ToClient) -> ToServer {
        // Do stuff on player turn
        info!("My turn!");
        sleep(Duration::from_millis(300));
        ToServer { new: None }
    }
}

pub struct Server {
    pub map: Map, // TODO
}

impl Server {
    pub fn update(&mut self, state: &ToServer) {
        // Current player's turn: do something here
        info!("Turn for player: {:?}", state);
    }

    pub fn turn(&mut self) {
        // Server player's turn: do something here
        info!("Turn for defender");
        sleep(Duration::from_millis(300));
    }
}
