mod cli;
mod metadata;
mod relations;
mod shared;
mod tags;
mod tree;
mod validate;

use cli::{parse_args, Command};
use shared::exit_with_error;

fn main() {
    let command = match parse_args() {
        Ok(command) => command,
        Err(message) => exit_with_error(&message),
    };

    match command {
        Command::Tree(config) => tree::run_tree(&config),
        Command::Metadata(config) => metadata::run_metadata(&config),
        Command::Tag(config) => tags::run_tag(&config),
        Command::Validate(config) => validate::run_validate(&config),
    }
}
