mod game;
mod net;

use clap::Parser;
pub use game::{Game, Point};
pub use net::{Client, MsgToClient, MsgToServer, Server};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
pub struct ServerCli {
    /// IP address of server
    pub address: String,
    /// Number of players
    pub players: usize,
    /// Number of guards
    pub guards: usize,
    /// Length of side of map
    pub len: usize,
}

#[derive(Parser)]
pub struct ClientCli {
    /// IP address of server
    pub address: String,
}
