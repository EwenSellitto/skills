use std::env;
use std::path::PathBuf;

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

pub fn parse_args() -> Result<Command, String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err(usage().to_string());
    };

    match command.as_str() {
        "tree" => Ok(Command::Tree(parse_command_args(args)?)),
        "metadata" => Ok(Command::Metadata(parse_command_args(args)?)),
        "tag" => Ok(Command::Tag(parse_tag_args(args)?)),
        "validate" => Ok(Command::Validate(parse_validate_args(args)?)),
        _ => Err(usage().to_string()),
    }
}

fn parse_command_args(args: impl Iterator<Item = String>) -> Result<Config, String> {
    let mut show_non_md = false;
    let mut positional = Vec::new();

    for arg in args {
        if arg == "--show-non-md" {
            show_non_md = true;
        } else {
            positional.push(arg);
        }
    }

    if positional.is_empty() || positional.len() > 2 {
        return Err(usage().to_string());
    }

    let max_depth = positional
        .get(1)
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| format!("Error: Invalid max_depth '{}'.", value))
        })
        .transpose()?
        .unwrap_or(2);

    Ok(Config {
        start_file: PathBuf::from(&positional[0]),
        max_depth,
        show_non_md,
    })
}

fn parse_tag_args(args: impl Iterator<Item = String>) -> Result<TagConfig, String> {
    let mut start_dir = PathBuf::from(".");
    let mut tags = Vec::new();
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        if arg == "--dir" {
            let Some(dir) = args.next() else {
                return Err(usage().to_string());
            };
            start_dir = PathBuf::from(dir);
            continue;
        }

        tags.push(arg);
    }

    Ok(TagConfig { tags, start_dir })
}

fn parse_validate_args(args: impl Iterator<Item = String>) -> Result<ValidateConfig, String> {
    let positional: Vec<String> = args.collect();
    if positional.len() > 1 {
        return Err(usage().to_string());
    }

    Ok(ValidateConfig {
        path: positional
            .first()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".")),
    })
}

fn usage() -> &'static str {
    "Usage:
  okf-cli tree [--show-non-md] <start_file.md> [max_depth (default 2)]
  okf-cli metadata [--show-non-md] <start_file.md> [max_depth (default 2)]
  okf-cli tag [--dir <start_dir>]
  okf-cli tag [--dir <start_dir>] <tag> [<tag> ...]
  okf-cli validate [path]"
}
