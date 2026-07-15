use std::path::{Component, Path, PathBuf};
use std::{fs, process};

pub fn resolve_start_dir(start_dir: &Path) -> PathBuf {
    match fs::canonicalize(start_dir) {
        Ok(path) if path.is_dir() => path,
        _ => exit_with_error(&format!("Error: Directory '{}' not found.", start_dir.display())),
    }
}

pub fn resolve_existing_path(path: &Path) -> PathBuf {
    match fs::canonicalize(path) {
        Ok(path) => path,
        Err(_) => exit_with_error(&format!("Error: Path '{}' not found.", path.display())),
    }
}

pub fn indent(depth: usize) -> String {
    " ".repeat(depth * 2)
}

pub fn print_leaf(depth: usize, marker: &str, text: &str) {
    println!("{indent}{marker}{text}", indent = indent(depth));
}

pub fn rel_path(path: &Path, root_dir: &Path) -> String {
    if let Ok(relative) = path.strip_prefix(root_dir) {
        return relative.display().to_string();
    }

    path.display().to_string()
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(Path::new("/")),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
        }
    }

    normalized
}

pub fn resolve_link_path(root_dir: &Path, dir: &Path, link: &str) -> PathBuf {
    let is_absolute = link.starts_with('/');
    let link = link.trim_start_matches('/');
    let base_dir = if is_absolute { root_dir } else { dir };
    normalize_path(&base_dir.join(link))
}

pub fn is_remote_link(link: &str) -> bool {
    matches!(
        link,
        _ if link.starts_with("http:")
            || link.starts_with("https:")
            || link.starts_with("mailto:")
            || link.starts_with("ftp:")
    )
}

pub fn is_markdown_path(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("md") | Some("markdown"))
}

pub fn trim_link_target(target: &str) -> &str {
    let target = target.trim();
    let target = target.strip_prefix('<').unwrap_or(target);
    let target = target.strip_suffix('>').unwrap_or(target);
    target.split('#').next().unwrap_or_default()
}

pub fn extract_markdown_links(content: &str) -> Vec<&str> {
    let bytes = content.as_bytes();
    let mut links = Vec::new();
    let mut index = 0;

    while index + 3 < bytes.len() {
        if bytes[index] == b']' && bytes[index + 1] == b'(' {
            let start = index + 2;
            if let Some(end) = find_link_end(bytes, start) {
                if let Some(link) = content.get(start..end) {
                    links.push(link);
                }
                index = end + 1;
                continue;
            }
        }
        index += 1;
    }

    links
}

pub fn extract_frontmatter_lines(content: &str) -> Vec<String> {
    extract_frontmatter_block(content)
        .ok()
        .flatten()
        .map(|block| block.lines().map(String::from).collect())
        .unwrap_or_default()
}

pub fn extract_frontmatter_block(content: &str) -> Result<Option<String>, &'static str> {
    let mut lines = content.lines();
    if lines.next() != Some("---") {
        return Ok(None);
    }

    let mut metadata = Vec::new();
    for line in lines {
        if line == "---" {
            return Ok(Some(metadata.join("\n")));
        }
        metadata.push(line);
    }

    Err("unterminated frontmatter")
}

pub fn exit_with_error(message: &str) -> ! {
    eprintln!("{message}");
    process::exit(1);
}

fn find_link_end(bytes: &[u8], start: usize) -> Option<usize> {
    if start >= bytes.len() {
        return None;
    }

    if bytes[start] == b'<' {
        let mut index = start + 1;
        while index < bytes.len() {
            if bytes[index] == b'>' {
                return bytes
                    .get(index + 1)
                    .copied()
                    .filter(|next| *next == b')')
                    .map(|_| index + 1);
            }
            index += 1;
        }
        return None;
    }

    let mut depth = 0u32;
    let mut index = start;
    while index < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' if depth == 0 => return Some(index),
            b')' => depth -= 1,
            _ => {}
        }
        index += 1;
    }

    None
}
