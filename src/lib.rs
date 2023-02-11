mod game;
mod net;
mod ui;

use clap::Parser;
pub use game::{Direction, Game, Point};
pub use net::{Client, MsgToClient, MsgToServer, Server};
pub use ui::{term::Terminal, UserInterface};

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
    /// Turn time (minutes)
    pub timer: u8,
}

#[derive(Parser)]
pub struct ClientCli {
    /// IP address of server
    pub address: String,
}
