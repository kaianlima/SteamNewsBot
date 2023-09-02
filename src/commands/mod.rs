pub mod news;
pub mod ping;

pub use anyhow::{Error, Result};

use crate::structs::Command;

pub fn commands() -> Vec<Command> {
    news::commands().into_iter()
        .chain(ping::commands())
        .collect()
}