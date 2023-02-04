use clap::Parser;
use hanzo::*;

fn main() -> Result<()> {
    let cli = ClientCli::parse();

    Client::new(&cli.address)?.run()
}
