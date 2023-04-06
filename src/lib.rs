mod config;
pub mod defaults;
mod game;
mod net;
mod ui;

use clap::Parser;
pub use config::Config;
pub use game::{Direction, Game, Point, Status, Tile};
pub use net::{Client, MsgToClient, MsgToServer, Server};
pub use ui::{term::Terminal, Colour, Key, UIBackend, UserInterface};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
pub struct Cli {
    /// IP address of server
    pub address: String,
}
