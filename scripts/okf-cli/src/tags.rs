use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::cli::TagConfig;
use crate::metadata::print_metadata_block;
use crate::shared::{extract_frontmatter_lines, is_markdown_path, rel_path, resolve_start_dir};

pub fn run_tag(config: &TagConfig) {
    let start_dir = resolve_start_dir(&config.start_dir);

    if config.tags.is_empty() {
        list_tags_in_dir(&start_dir);
        return;
    }

    let mut matches = Vec::new();
    let wanted_tags: HashSet<String> = config.tags.iter().map(|tag| normalize_tag(tag)).collect();

    collect_tag_matches_in_dir(&start_dir, &start_dir, &wanted_tags, &mut matches);

    if matches.is_empty() {
        println!("No entries found for tags: {}.", config.tags.join(", "));
        return;
    }

    for (index, (rel, metadata_lines)) in matches.into_iter().enumerate() {
        if index > 0 {
            println!();
        }
        print_metadata_block(0, &rel, metadata_lines);
    }
}

fn list_tags_in_dir(start_dir: &Path) {
    let mut tags = HashSet::new();
    collect_all_tags_in_dir(start_dir, &mut tags);

    if tags.is_empty() {
        println!("No tags found.");
        return;
    }

    let mut sorted_tags: Vec<String> = tags.into_iter().collect();
    sorted_tags.sort();

    for tag in sorted_tags {
        println!("{tag}");
    }
}

fn collect_tag_matches_in_dir(
    dir: &Path,
    root_dir: &Path,
    wanted_tags: &HashSet<String>,
    matches: &mut Vec<(String, Vec<String>)>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_tag_matches_in_dir(&path, root_dir, wanted_tags, matches);
            continue;
        }

        if !is_markdown_path(&path) {
            continue;
        }

        collect_tag_match(&path, root_dir, wanted_tags, matches);
    }
}

fn collect_all_tags_in_dir(dir: &Path, tags: &mut HashSet<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_all_tags_in_dir(&path, tags);
            continue;
        }

        if !is_markdown_path(&path) {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        for tag in extract_tags(&content) {
            let normalized = normalize_tag(&tag);
            if !normalized.is_empty() {
                tags.insert(normalized);
            }
        }
    }
}

fn collect_tag_match(
    file: &Path,
    root_dir: &Path,
    wanted_tags: &HashSet<String>,
    matches: &mut Vec<(String, Vec<String>)>,
) {
    if !file.is_file() {
        return;
    }

    let content = match fs::read_to_string(file) {
        Ok(content) => content,
        Err(_) => return,
    };

    if has_any_tag(&content, wanted_tags) {
        matches.push((rel_path(file, root_dir), extract_frontmatter_lines(&content)));
    }
}

fn has_any_tag(content: &str, wanted_tags: &HashSet<String>) -> bool {
    extract_tags(content)
        .into_iter()
        .map(|tag| normalize_tag(&tag))
        .any(|tag| wanted_tags.contains(&tag))
}

fn extract_tags(content: &str) -> Vec<String> {
    let lines = extract_frontmatter_lines(content);
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index].trim();
        let Some(rest) = line.strip_prefix("tags:") else {
            index += 1;
            continue;
        };

        let rest = rest.trim();
        if rest.is_empty() {
            return extract_block_tags(&lines, index + 1);
        }

        if rest.starts_with('[') && rest.ends_with(']') {
            return rest[1..rest.len() - 1]
                .split(',')
                .map(clean_tag)
                .filter(|tag| !tag.is_empty())
                .collect();
        }

        let tag = clean_tag(rest);
        return if tag.is_empty() { Vec::new() } else { vec![tag] };
    }

    Vec::new()
}

fn extract_block_tags(lines: &[String], start_index: usize) -> Vec<String> {
    let mut tags = Vec::new();

    for line in &lines[start_index..] {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Some(tag) = trimmed.strip_prefix('-') else {
            break;
        };

        let tag = clean_tag(tag);
        if !tag.is_empty() {
            tags.push(tag);
        }
    }

    tags
}

fn clean_tag(tag: &str) -> String {
    tag.trim()
        .trim_start_matches('#')
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn normalize_tag(tag: &str) -> String {
    clean_tag(tag).to_ascii_lowercase()
}
