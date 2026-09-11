use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub enum Command {
    Tree(Config),
    Metadata(Config),
    Tag(TagConfig),
    Validate(ValidateConfig),
}

pub struct Config {
    pub start_file: PathBuf,
    pub max_depth: usize,
    pub show_non_md: bool,
}

pub struct TagConfig {
    pub tags: Vec<String>,
    pub start_dir: PathBuf,
}

pub struct ValidateConfig {
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
#[command(name = "okf-cli", version, about = "OKF knowledge base CLI")]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    Tree {
        #[arg(long)]
        show_non_md: bool,
        start_file: PathBuf,
        #[arg(default_value_t = 2)]
        max_depth: usize,
    },
    Metadata {
        #[arg(long)]
        show_non_md: bool,
        start_file: PathBuf,
        #[arg(default_value_t = 2)]
        max_depth: usize,
    },
    Tag {
        #[arg(long, default_value = ".")]
        dir: PathBuf,
        tags: Vec<String>,
    },
    Validate {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

pub fn parse_args() -> Command {
    let cli = Cli::parse();
    match cli.command {
        CliCommand::Tree {
            start_file,
            max_depth,
            show_non_md,
        } => Command::Tree(Config {
            start_file,
            max_depth,
            show_non_md,
        }),
        CliCommand::Metadata {
            start_file,
            max_depth,
            show_non_md,
        } => Command::Metadata(Config {
            start_file,
            max_depth,
            show_non_md,
        }),
        CliCommand::Tag { tags, dir } => Command::Tag(TagConfig {
            tags,
            start_dir: dir,
        }),
        CliCommand::Validate { path } => Command::Validate(ValidateConfig { path }),
    }
}
