use clap::Parser;
use hanzo::*;

fn main() {
    let cli = ClientCli::parse();
    Terminal::new()
        .and_then(|ui| Client::new(&cli.address, ui))
        .and_then(|mut client| client.run())
        .unwrap_or_else(|err| eprintln!("Something went wrong: \"{}\"", err));
}
