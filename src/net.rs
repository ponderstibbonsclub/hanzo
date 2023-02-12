use crate::{Direction, Game, Point, Result, Status, UserInterface};
use bincode::{deserialize_from, serialize_into};
use log::{info, LevelFilter};
use serde::{Deserialize, Serialize};
use simple_logging::log_to_stderr;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{spawn, JoinHandle};

#[derive(Serialize, Deserialize, Debug)]
/// Information sent from server to client each turn
pub struct MsgToClient {
    // Is it the player's turn?
    pub turn: bool,
    // Is the player the defender?
    pub defender: bool,
    // Players' positions (if alive)
    pub positions: Vec<Option<Point>>,
    // Guards' positions (if alive)
    pub guards: Vec<Option<(Point, Direction)>>,
    // Game finished?
    pub quit: Status,
}

#[derive(Serialize, Deserialize, Debug)]
/// Information sent from client to server each turn
pub struct MsgToServer {
    // New position of player's character
    pub new: Option<Point>,
    // New positions of guards
    pub guards: Vec<Option<(Point, Direction)>>,
    // Game finished?
    pub quit: Status,
}

/// A client connection
pub struct ClientThread {
    stream: TcpStream,
    tx: Sender<MsgToServer>,
    rx: Receiver<MsgToClient>,
}

impl ClientThread {
    pub fn run(&self, game: Game) -> Result<()> {
        info!("Client connected!");

        // Send initial game state
        serialize_into(&self.stream, &game)?;

        loop {
            // Send latest info to client
            let msg = self.rx.recv()?;
            serialize_into(&self.stream, &msg)?;
            if msg.quit != Status::Running {
                break;
            }

            // Receive update if it's the player's turn
            if msg.turn {
                let msg = deserialize_from(&self.stream)?;
                self.tx.send(msg)?;
            }
        }

        Ok(())
    }
}

/// A client handle
pub struct ClientHandle {
    pub handle: JoinHandle<()>,
    pub tx: Sender<MsgToClient>,
    pub rx: Receiver<MsgToServer>,
}

impl ClientHandle {
    pub fn new(listener: &TcpListener, game: Game) -> Result<Self> {
        let stream = listener.accept()?.0;

        let (tx0, rx0): (Sender<MsgToClient>, Receiver<MsgToClient>) = channel();
        let (tx1, rx1): (Sender<MsgToServer>, Receiver<MsgToServer>) = channel();

        let client = ClientThread {
            stream,
            tx: tx1,
            rx: rx0,
        };
        let handle = spawn(move || client.run(game).unwrap());

        Ok(ClientHandle {
            handle,
            tx: tx0,
            rx: rx1,
        })
    }
}

/// A server to manage client connections
pub struct Server {
    //listener: TcpListener,
    clients: Vec<ClientHandle>,
    game: Game,
}

impl Server {
    pub fn new(game: Game) -> Result<Self> {
        log_to_stderr(LevelFilter::Info);
        info!("Address: {}, players: {}", game.address, game.players);

        let mut clients = Vec::with_capacity(game.players);
        let listener = TcpListener::bind(&game.address)?;
        for i in 0..game.players {
            let mut g = game.clone();
            g.player = i;
            clients.push(ClientHandle::new(&listener, g)?);
        }

        Ok(Server { clients, game })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut current: usize = 0;
        loop {
            // Check victory conditions
            self.game.victory();

            // Send updates to clients
            for (i, client) in self.clients.iter().enumerate() {
                let msg = self.game.turn(i, current);
                client.tx.send(msg)?;
            }
            info!("Update broadcasted to clients");

            if self.game.quit != Status::Running {
                break;
            }

            // Receive update from current player's client
            let msg = self.clients[current].rx.recv()?;
            info!("Received update from player {}", current);
            self.game.update(msg, current);

            current = (current + 1) % self.game.players;
        }

        for client in self.clients.drain(0..) {
            client.handle.join().unwrap();
        }

        Ok(())
    }
}

/// A player client
pub struct Client<T: UserInterface> {
    stream: TcpStream,
    game: Game,
    ui: T,
}

impl<T: UserInterface> Client<T> {
    pub fn new(address: &str, ui: T) -> Result<Self> {
        log_to_stderr(LevelFilter::Info);
        //log_to_file("hanzo.log", LevelFilter::Error)?;

        let stream = TcpStream::connect(address)?;
        let game = deserialize_from(&stream)?;
        info!("Connected to {}. Waiting for server...", address);

        Ok(Client { stream, game, ui })
    }

    pub fn run(&mut self) -> Result<()> {
        let quit: Status;
        loop {
            // Receive update from server
            let msg: MsgToClient = deserialize_from(&self.stream)?;
            self.game.display(&mut self.ui, &msg)?;
            if msg.quit != Status::Running {
                quit = msg.quit;
                break;
            }

            // Send back update if it's our turn
            if msg.turn {
                let msg = self.game.play(msg.defender, &mut self.ui)?;
                serialize_into(&self.stream, &msg)?;
            }
        }
        self.ui.reset();

        if quit != Status::Quit {
            println!("\n\nCongratulations! You win.\n");
        } else {
            println!("\n\nGame over! Thanks for playing.\n");
        }

        Ok(())
    }
}
