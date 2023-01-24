use clap::Parser;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Tile {
    Floor,
    Wall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Map(pub Vec<Tile>);

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
