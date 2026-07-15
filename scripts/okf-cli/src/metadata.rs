use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::Config;
use crate::relations::{extract_relations, Relation};
use crate::shared::{
    extract_frontmatter_lines, indent, print_leaf, rel_path, resolve_existing_path,
};

pub fn run_metadata(config: &Config) {
    let start_file = resolve_existing_path(&config.start_file);
    let root_dir = start_file.parent().unwrap_or(Path::new(".")).to_path_buf();
    let mut branch = Vec::new();
    print_metadata(
        &start_file,
        0,
        &mut branch,
        &root_dir,
        config.max_depth,
        config.show_non_md,
    );
}

pub fn print_metadata_block(depth: usize, rel: &str, metadata_lines: Vec<String>) {
    let indent = indent(depth);
    println!("{indent}{rel}");

    if metadata_lines.is_empty() {
        println!("{indent}  metadata: none");
        return;
    }

    println!("{indent}  metadata:");
    for line in metadata_lines {
        println!("{indent}    {line}");
    }
}

fn print_metadata(
    file: &Path,
    depth: usize,
    branch: &mut Vec<PathBuf>,
    root_dir: &Path,
    max_depth: usize,
    show_non_md: bool,
) {
    let rel = rel_path(file, root_dir);

    if !file.is_file() {
        print_leaf(depth, "! ", &rel);
        return;
    }

    if branch.iter().any(|ancestor| ancestor == file) {
        print_leaf(depth, "~ ", &rel);
        return;
    }

    let content = match fs::read_to_string(file) {
        Ok(content) => content,
        Err(_) => {
            print_leaf(depth, "! ", &rel);
            return;
        }
    };

    print_metadata_block(depth, &rel, extract_frontmatter_lines(&content));

    if depth >= max_depth {
        return;
    }

    let dir = file.parent().unwrap_or(root_dir);
    branch.push(file.to_path_buf());

    for relation in extract_relations(&content, dir, root_dir, show_non_md) {
        match relation {
            Relation::Markdown(next_file) => {
                print_metadata(&next_file, depth + 1, branch, root_dir, max_depth, show_non_md)
            }
            Relation::Leaf { text, broken } => {
                let indent = indent(depth + 1);
                let relation_kind = if broken { "broken" } else { "leaf" };
                println!("{indent}relation: {text} [{relation_kind}]");
            }
        }
    }

    branch.pop();
}
