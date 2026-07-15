use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::Config;
use crate::relations::{extract_relations, Relation};
use crate::shared::{print_leaf, rel_path, resolve_existing_path};

pub fn run_tree(config: &Config) {
    let start_file = resolve_existing_path(&config.start_file);
    let root_dir = start_file.parent().unwrap_or(Path::new(".")).to_path_buf();
    let mut branch = Vec::new();
    build_tree(
        &start_file,
        0,
        &mut branch,
        &root_dir,
        config.max_depth,
        config.show_non_md,
    );
}

fn build_tree(
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

    print_leaf(depth, "", &rel);

    if depth >= max_depth {
        return;
    }

    let content = match fs::read_to_string(file) {
        Ok(content) => content,
        Err(_) => return,
    };

    let dir = file.parent().unwrap_or(root_dir);
    branch.push(file.to_path_buf());

    for relation in extract_relations(&content, dir, root_dir, show_non_md) {
        match relation {
            Relation::Markdown(next_file) => {
                build_tree(&next_file, depth + 1, branch, root_dir, max_depth, show_non_md)
            }
            Relation::Leaf { text, broken } => {
                let marker = if broken { "! " } else { "" };
                print_leaf(depth + 1, marker, &text);
            }
        }
    }

    branch.pop();
}
