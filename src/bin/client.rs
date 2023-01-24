use clap::Parser;
use hanzo::*;
use log::{error, info, LevelFilter};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let cli = ClientCli::parse();

    simple_logging::log_to_stderr(LevelFilter::Info);

    let stream = TcpStream::connect(cli.address)?;
    let map: Map = bincode::deserialize_from(&stream).expect("Something went wrong receiving map!");
    let mut player = Player { map };

    info!("Connected! Waiting for server...");
    loop {
        // Receive latest state from server
        match bincode::deserialize_from::<&TcpStream, ToClient>(&stream) {
            Ok(state) => {
                player.display(&state);
                if state.turn {
                    let out = player.turn(&state);

                    // Send back state when finished
                    if let Err(e) = bincode::serialize_into(&stream, &out) {
                        error!("Something went wrong! {:?}", e);
                        break;
                    }
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
