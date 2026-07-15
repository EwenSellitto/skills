use std::path::{Path, PathBuf};

use crate::shared::{
    extract_markdown_links, is_markdown_path, is_remote_link, rel_path, resolve_link_path,
    trim_link_target,
};

pub enum Relation {
    Markdown(PathBuf),
    Leaf { text: String, broken: bool },
}

pub fn extract_relations(content: &str, dir: &Path, root_dir: &Path, show_non_md: bool) -> Vec<Relation> {
    let mut relations = Vec::new();

    for raw_link in extract_markdown_links(content) {
        let link = trim_link_target(raw_link);
        if link.is_empty() {
            continue;
        }

        if is_remote_link(link) {
            if show_non_md {
                relations.push(Relation::Leaf {
                    text: link.to_string(),
                    broken: false,
                });
            }
            continue;
        }

        let next_file = resolve_link_path(root_dir, dir, link);
        if is_markdown_path(&next_file) {
            relations.push(Relation::Markdown(next_file));
            continue;
        }

        if show_non_md {
            let next_rel = rel_path(&next_file, root_dir);
            relations.push(Relation::Leaf {
                text: next_rel,
                broken: !next_file.is_file(),
            });
        }
    }

    relations
}
