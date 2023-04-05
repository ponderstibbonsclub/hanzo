use crate::defaults;
use log::info;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::time::Duration;
use toml::from_str;

#[derive(Deserialize, Debug)]
struct TomlConfig {
    input_timeout: Option<u64>,
    attacker_actions: Option<u8>,
    defender_actions: Option<u8>,
    detection_actions: Option<u8>,
    viewcone_length: Option<u8>,
    viewcone_width: Option<u8>,
    turn_time: Option<u8>,
    players: Option<u8>,
    num_guards: Option<u8>,
    len: Option<u8>,
}

/// Assorted configuration options (defined server-side)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    /// Timeout in milliseconds waiting for user input
    pub input_timeout: u64,
    /// Number of actions for an attacker each turn
    pub attacker_actions: isize,
    /// Number of actions for a defender each turn
    pub defender_actions: isize,
    /// Number of actions inside view cone before detection
    pub detection_actions: isize,
    /// Length of viewcone
    pub viewcone_length: i16,
    /// Half-width of viewcone
    pub viewcone_width: usize,
    /// Time in seconds per turn
    pub turn_time: Duration,
    /// Number of players
    pub players: usize,
    /// Number of guards
    pub num_guards: usize,
    /// Length of side of map
    pub len: usize,
}

impl Default for Config {
    /// Currently use test parameters
    fn default() -> Self {
        Config {
            input_timeout: 300,
            attacker_actions: 5,
            defender_actions: 10,
            detection_actions: 3,
            viewcone_length: 16,
            viewcone_width: 10,
            turn_time: Duration::from_secs(120),
            players: defaults::PLAYERS,
            num_guards: 5,
            len: 48,
        }
    }
}

impl Config {
    /// Read configuration from "hanzo.toml" or use defaults
    pub fn new() -> Config {
        let mut conf = Config::default();
        if let Ok(file) = read_to_string("hanzo.toml") {
            info!("Configuration read from hanzo.toml");
            if let Ok(toml) = from_str::<TomlConfig>(&file) {
                macro_rules! choose_value {
                    ($attr:ident, $type:ty) => {
                        if let Some($attr) = toml.$attr {
                            conf.$attr = $attr as $type;
                        }
                    };
                }
                choose_value!(input_timeout, u64);
                choose_value!(attacker_actions, isize);
                choose_value!(defender_actions, isize);
                choose_value!(detection_actions, isize);
                choose_value!(viewcone_length, i16);
                choose_value!(viewcone_width, usize);
                choose_value!(players, usize);
                choose_value!(num_guards, usize);
                choose_value!(len, usize);
                if let Some(turn_time) = toml.turn_time {
                    conf.turn_time = Duration::from_secs((turn_time * 60).into());
                }
            }
        } else {
            info!("hanzo.toml not found");
        }
        conf
    }
}
