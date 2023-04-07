A prototype multiplayer stealth strategy game.

## Usage

To build from source you'll need to install [Rust](https://www.rust-lang.org/tools/install).

```sh
# Edit server configuration file as needed
ed hanzo.toml

# Run the server locally on port 5000
cargo run --bin server 127.0.0.1:5000

# Connect a client
cargo run --bin client 127.0.0.1:5000
```

You might also be able to use a binary from the
[releases](https://github.com/ponderstibbonsclub/hanzo/releases)
page:
- Windows x86_64 `client.exe`
- Linux x86_64 `server` and `client`

See [INTRO](./INTRO.md) for an introduction to the game.

## License

[MIT](./LICENSE)
