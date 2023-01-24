use clap::Parser;
use hanzo::*;
use log::{error, info, LevelFilter};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

// Dedicated worker thread for sending and receiving game state with a
// particular client
fn handle_client(stream: TcpStream, map: Map, tx: Sender<ToServer>, rx: Receiver<ToClient>) {
    info!("Client connected!");

    // Send map to client
    if let Err(e) = bincode::serialize_into(&stream, &map) {
        error!("Something went wrong! {:?}", e);
        return;
    }

    while let Ok(out) = rx.recv() {
        // Send latest game info to client
        if let Err(e) = bincode::serialize_into(&stream, &out) {
            error!("Something went wrong! {:?}", e);
            break;
        }

        // If it's this player's turn then wait for their update
        if out.turn {
            match bincode::deserialize_from(&stream) {
                Ok(msg) => {
                    if let Err(e) = tx.send(msg) {
                        error!("Something went wrong! {:?}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Something went wrong! {:?}", e);
                    break;
                }
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let cli = ServerCli::parse();

    simple_logging::log_to_stderr(LevelFilter::Info);

    // Generate map procedurally...
    let map = Map(vec![Tile::Floor; GRID_SIZE * GRID_SIZE]);

    let listener = TcpListener::bind("127.0.0.1:50000")?;
    let mut handles = Vec::new();
    let mut channels = Vec::new();

    // Spawn new thread for each player connection
    for _ in 0..cli.players {
        let stream = listener.accept()?.0;
        let thread_map = map.clone();

        // Create channels for communicating with client threads
        let (tx0, rx0): (Sender<ToClient>, Receiver<ToClient>) = mpsc::channel();
        let (tx1, rx1): (Sender<ToServer>, Receiver<ToServer>) = mpsc::channel();
        channels.push((tx0, rx1));

        let handle = thread::spawn(move || {
            handle_client(stream, thread_map, tx1, rx0);
        });
        handles.push(handle);
    }

    // Process round of turns
    let mut current: usize = 0;
    loop {
        // Send latest game info to all clients
        for (i, channel) in channels.iter().enumerate() {
            let turn = i == current;
            let out = ToClient {
                turn,
                pos: None,
                guards: [None; NUM_GUARDS],
            };
            if let Err(e) = channel.0.send(out) {
                error!("Something went wrong! {:?}", e);
                break;
            }
        }

        if let Ok(msg) = channels[current].1.recv() {
            // Current player's turn: do something here
            info!("Turn for player {}: {:?}", current, msg);
        } else {
            break;
        }

        current += 1;
        if current == cli.players {
            // Server player's turn: do something here
            info!("Turn for defender");
            sleep(Duration::from_millis(300));
            current = 0;
        }
    }

    // Cleanup
    for handle in handles.into_iter() {
        handle.join().unwrap();
    }

    Ok(())
}
