mod game;
mod net;
mod ui;

use clap::Parser;
pub use game::{Game, Point};
pub use net::{Client, MsgToClient, MsgToServer, Server};
pub use ui::{term::Terminal, UserInterface};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
pub struct ServerCli {
    /// IP address of server
    pub address: String,
    /// Number of players
    #[arg(default_value_t = 4)]
    pub players: usize,
    /// Number of guards
    #[arg(default_value_t = 5)]
    pub guards: usize,
    #[arg(default_value_t = 48)]
    /// Length of side of map
    pub len: usize,
    /// Turn time (minutes)
    #[arg(default_value_t = 2)]
    pub timer: u8,
}

#[derive(Parser)]
pub struct ClientCli {
    /// IP address of server
    pub address: String,
}
