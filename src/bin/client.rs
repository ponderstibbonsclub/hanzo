use clap::Parser;
use hanzo::*;
use log::{error, info, LevelFilter};
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let cli = ClientCli::parse();

    simple_logging::log_to_stderr(LevelFilter::Info);

    let stream = TcpStream::connect(cli.address)?;
    let map: Map = bincode::deserialize_from(&stream).expect("Something went wrong receiving map!");

    info!("Connected! Waiting for server...");
    loop {
        // Receive latest state from server
        match bincode::deserialize_from::<&TcpStream, ToClient>(&stream) {
            Ok(state) => {
                if state.turn {
                    // Do stuff on player turn
                    info!("My turn!");
                    sleep(Duration::from_millis(300));

                    // Send back state when finished
                    let out = ToServer { new: None };
                    if let Err(e) = bincode::serialize_into(&stream, &out) {
                        error!("Something went wrong! {:?}", e);
                        break;
                    }
                } else {
                    // Display any updates
                    info!("Someone else's turn!");
                }
            }
            Err(e) => {
                error!("Something went wrong! {:?}", e);
                break;
            }
        }
    }
    Ok(())
}
