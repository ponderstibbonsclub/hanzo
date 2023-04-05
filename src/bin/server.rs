use clap::Parser;
use hanzo::*;
use log::LevelFilter;
use simple_logging::log_to_stderr;

fn main() -> Result<()> {
    log_to_stderr(LevelFilter::Info);
    let cli = Cli::parse();
    let game = Game::new(cli);

    Server::new(game)?.run()
}
