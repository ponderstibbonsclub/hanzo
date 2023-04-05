use clap::Parser;
use hanzo::*;
use log::LevelFilter;
use simple_logging::log_to_stderr;

fn main() {
    log_to_stderr(LevelFilter::Info);
    let cli = Cli::parse();
    Terminal::new()
        .and_then(|ui| Client::new(&cli.address, UserInterface::new(ui)))
        .and_then(|mut client| client.run())
        .unwrap_or_else(|err| eprintln!("Something went wrong: \"{err}\""));
}
