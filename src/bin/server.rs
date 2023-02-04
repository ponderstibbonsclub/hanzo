use clap::Parser;
use hanzo::*;

fn main() -> Result<()> {
    let cli = ServerCli::parse();
    let game = Game::new(cli);

    Server::new(game)?.run()
}
