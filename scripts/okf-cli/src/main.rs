mod cli;
mod metadata;
mod relations;
mod shared;
mod tags;
mod tree;
mod validate;

use cli::{Command, parse_args};

fn main() {
    let command = parse_args();

    match command {
        Command::Tree(config) => tree::run_tree(&config),
        Command::Metadata(config) => metadata::run_metadata(&config),
        Command::Tag(config) => tags::run_tag(&config),
        Command::Validate(config) => validate::run_validate(&config),
    }
}
