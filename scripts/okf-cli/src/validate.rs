use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

use crate::cli::ValidateConfig;
use crate::shared::{
    extract_frontmatter_block, extract_markdown_links, is_markdown_path, is_remote_link,
    rel_path, resolve_existing_path, resolve_link_path, trim_link_target,
};

pub fn run_validate(config: &ValidateConfig) {
    let root_dir = resolve_validate_root(&config.path);
    let files = collect_markdown_files(&root_dir);
    let mut errors = Vec::new();

    if !root_dir.join("index.md").is_file() {
        errors.push("root: missing index.md".to_string());
    }

    if !root_dir.join("log.md").is_file() {
        errors.push("root: missing log.md".to_string());
    }

    let mut graph: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

    for file in &files {
        let content = match fs::read_to_string(file) {
            Ok(content) => content,
            Err(_) => {
                errors.push(format!("{}: unreadable", rel_path(file, &root_dir)));
                continue;
            }
        };

        validate_file(file, &root_dir, &content, &mut errors);
        graph.insert(file.clone(), collect_markdown_links(file, &root_dir, &content, &mut errors));
    }

    errors.extend(find_cycles(&graph, &root_dir));
    errors.extend(find_unreachable(&root_dir, &files, &graph));
    errors.sort();
    errors.dedup();

    if errors.is_empty() {
        println!("OK");
        return;
    }

    println!("ERR {}", errors.len());
    for error in errors {
        println!("{error}");
    }
}

fn resolve_validate_root(path: &Path) -> PathBuf {
    let path = resolve_existing_path(path);
    if path.is_dir() {
        return path;
    }

    let start_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut current = Some(start_dir);
    while let Some(dir) = current {
        if dir.join("index.md").is_file() {
            return dir.to_path_buf();
        }
        let parent = dir.parent();
        if parent == Some(dir) {
            break;
        }
        current = parent;
    }

    start_dir.to_path_buf()
}

fn collect_markdown_files(root_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    visit_markdown_files(root_dir, &mut files);
    files.sort();
    files
}

fn visit_markdown_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_markdown_files(&path, files);
            continue;
        }

        if is_markdown_path(&path) {
            files.push(path);
        }
    }
}

fn validate_file(file: &Path, root_dir: &Path, content: &str, errors: &mut Vec<String>) {
    let rel = rel_path(file, root_dir);
    let file_name = file.file_name().and_then(|name| name.to_str()).unwrap_or("");
    let is_root_index = file == root_dir.join("index.md");

    match file_name {
        "index.md" | "log.md" => validate_reserved_file(&rel, file_name, is_root_index, content, errors),
        _ => validate_concept_file(&rel, content, errors),
    }
}

fn validate_reserved_file(
    rel: &str,
    file_name: &str,
    is_root_index: bool,
    content: &str,
    errors: &mut Vec<String>,
) {
    let frontmatter = match extract_frontmatter_block(content) {
        Ok(frontmatter) => frontmatter,
        Err(message) => {
            errors.push(format!("{rel}: {message}"));
            return;
        }
    };

    let Some(frontmatter) = frontmatter else {
        return;
    };

    if is_root_index {
        validate_root_index_frontmatter(rel, &frontmatter, errors);
        return;
    }

    errors.push(format!("{rel}: {file_name} must not have frontmatter"));
}

fn validate_root_index_frontmatter(rel: &str, frontmatter: &str, errors: &mut Vec<String>) {
    let value = match serde_yaml::from_str::<Value>(frontmatter) {
        Ok(value) => value,
        Err(_) => {
            errors.push(format!("{rel}: invalid YAML"));
            return;
        }
    };

    let Some(map) = value.as_mapping() else {
        errors.push(format!("{rel}: root index frontmatter must be a map"));
        return;
    };

    if map.len() != 1 {
        errors.push(format!("{rel}: root index frontmatter may only contain okf_version"));
        return;
    }

    let key = Value::String("okf_version".to_string());
    match map.get(&key).and_then(Value::as_str) {
        Some("0.1") => {}
        _ => errors.push(format!("{rel}: okf_version must be \"0.1\"")),
    }
}

fn validate_concept_file(rel: &str, content: &str, errors: &mut Vec<String>) {
    let frontmatter = match extract_frontmatter_block(content) {
        Ok(Some(frontmatter)) => frontmatter,
        Ok(None) => {
            errors.push(format!("{rel}: missing frontmatter"));
            return;
        }
        Err(message) => {
            errors.push(format!("{rel}: {message}"));
            return;
        }
    };

    let value = match serde_yaml::from_str::<Value>(&frontmatter) {
        Ok(value) => value,
        Err(_) => {
            errors.push(format!("{rel}: invalid YAML"));
            return;
        }
    };

    let Some(map) = value.as_mapping() else {
        errors.push(format!("{rel}: frontmatter must be a map"));
        return;
    };

    let key = Value::String("type".to_string());
    match map.get(&key).and_then(Value::as_str).map(str::trim) {
        Some(type_name) if !type_name.is_empty() => {}
        _ => errors.push(format!("{rel}: missing type")),
    }
}

fn collect_markdown_links(
    file: &Path,
    root_dir: &Path,
    content: &str,
    errors: &mut Vec<String>,
) -> Vec<PathBuf> {
    let dir = file.parent().unwrap_or(root_dir);
    let rel = rel_path(file, root_dir);
    let mut targets = BTreeSet::new();

    for raw_link in extract_markdown_links(content) {
        let link = trim_link_target(raw_link);
        if link.is_empty() || is_remote_link(link) {
            continue;
        }

        let target = resolve_link_path(root_dir, dir, link);
        if !is_markdown_path(&target) {
            continue;
        }

        if !target.is_file() {
            errors.push(format!("{rel} -> {link}: broken"));
            continue;
        }

        targets.insert(target);
    }

    targets.into_iter().collect()
}

fn find_cycles(graph: &HashMap<PathBuf, Vec<PathBuf>>, root_dir: &Path) -> Vec<String> {
    let mut errors = BTreeSet::new();
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();

    for node in graph.keys() {
        visit_cycle(node, graph, root_dir, &mut visiting, &mut visited, &mut errors);
    }

    errors.into_iter().collect()
}

fn visit_cycle(
    node: &PathBuf,
    graph: &HashMap<PathBuf, Vec<PathBuf>>,
    root_dir: &Path,
    visiting: &mut HashSet<PathBuf>,
    visited: &mut HashSet<PathBuf>,
    errors: &mut BTreeSet<String>,
) {
    if visited.contains(node) {
        return;
    }

    visiting.insert(node.clone());

    if let Some(targets) = graph.get(node) {
        for target in targets {
            if visiting.contains(target) {
                errors.insert(format!(
                    "{} -> {}: cycle",
                    rel_path(node, root_dir),
                    rel_path(target, root_dir)
                ));
                continue;
            }

            visit_cycle(target, graph, root_dir, visiting, visited, errors);
        }
    }

    visiting.remove(node);
    visited.insert(node.clone());
}

fn find_unreachable(
    root_dir: &Path,
    files: &[PathBuf],
    graph: &HashMap<PathBuf, Vec<PathBuf>>,
) -> Vec<String> {
    let start = root_dir.join("index.md");
    if !start.is_file() {
        return Vec::new();
    }

    let mut reachable = HashSet::new();
    mark_reachable(&start, graph, &mut reachable);

    let mut errors = Vec::new();
    for file in files {
        if file == &root_dir.join("index.md") {
            continue;
        }

        if matches!(file.file_name().and_then(|name| name.to_str()), Some("index.md") | Some("log.md")) {
            continue;
        }

        if reachable.contains(file) {
            continue;
        }

        errors.push(format!("{}: unreachable from index.md", rel_path(file, root_dir)));
    }

    errors
}

fn mark_reachable(node: &PathBuf, graph: &HashMap<PathBuf, Vec<PathBuf>>, reachable: &mut HashSet<PathBuf>) {
    if !reachable.insert(node.clone()) {
        return;
    }

    if let Some(targets) = graph.get(node) {
        for target in targets {
            mark_reachable(target, graph, reachable);
        }
    }
}
