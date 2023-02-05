pub mod term;

use crate::{Game, Result};

pub trait UserInterface {
    /// Display the current game state
    fn display(&mut self, game: &Game, defender: bool) -> Result<()>;

    /// Display a message
    fn message(&mut self, msg: &str) -> Result<()>;

    /// Process user input
    fn input(&mut self, game: &mut Game, defender: bool) -> Result<()>;

    /// Clean-up user interface
    fn reset(&mut self);
}
