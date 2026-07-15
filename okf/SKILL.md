---
name: okf
description: OKF bundle — create, validate, enrich, or convert Open Knowledge Format knowledge bundles for AI agent consumption. Use when the user wants to build a Markdown knowledge base, check OKF conformance or link-graph integrity, add schema/citations/examples to docs, or import from Notion, Obsidian, or CSV.
---

# Open Knowledge Format (OKF)

Vendor-neutral spec: a directory tree of Markdown docs with YAML frontmatter, tuned for LLM consumption. Full rules live in `references/SPEC.md`; this file is the working reference plus the workflow.

## Concepts

* **Bundle**: a directory tree of `.md` files (Git repo, tarball, or folder).
* **Concept**: one `.md` file. Its ID is its path minus the `.md` suffix (e.g. `tables/users`).
* **Frontmatter**: YAML, required in every concept file. `type` is the only required field — a free-form string used as the route/filter target. Recommended: `title`, `description`, `resource`, `tags`, `timestamp`. Unknown fields are allowed; never delete or reject them.
* **Links**: standard Markdown links. Absolute paths (`/tables/users.md`) preferred, relative allowed. Broken links permitted but flagged by tooling. They form untyped graph edges.
* **Reserved files**: `index.md` (nav listing, no frontmatter; the root `index.md` may carry `okf_version: "0.1"`) and `log.md` (newest-first history with ISO 8601 date headings, no frontmatter).
* **Citations**: numbered external links at the bottom of a doc.
* **Conformance**: parseable YAML with a present `type` field. That permissive bar lets a bundle grow without breakage.

## Conventional headings

* `# Schema` — column/field tables for data assets.
* `# Examples` — code or query blocks.
* `# Citations` — numbered external sources.

## Scripts

`okf-cli validate [path]` — validates the OKF structure for a bundle and prints either `OK` or a compact `ERR <count>` list. Omit `path` for the current directory. If `path` is a file, validation runs from the nearest ancestor bundle root with `index.md`, otherwise from the file's parent directory. Use this first for conformance and link-graph checks.

`okf-cli tree <start_file.md> [max_depth]` — walks Markdown links from a file and prints an indented tree. `!` marks a broken link, `~` marks a cycle, and `--show-non-md` reveals non-`.md`/remote links as leaves (never parsed further). Use it to confirm the link graph is connected and free of unintended `!`/`~` before presenting a bundle.

```
index.md
  tables/users.md
    tables/orders.md
      ~ tables/users.md
  metrics/dau.md
    ! metrics/broken-ref.md
    https://example.com
    assets/logo.png
```

`okf-cli tag [--dir <start_dir>] [<tag> ...]` — with one or more tags, lists entries whose frontmatter `tags` include any of them; with none, lists the unique tags found under the directory.

## Workflow

### 1. Create
1. Choose a domain-based directory structure.
2. Author concepts: YAML frontmatter (with `type`) + standard Markdown body.
3. Cross-link concepts using absolute paths where possible.
4. Generate `index.md` and `log.md`.

*Completion*: every concept file parses, every one has a `type`, `index.md` and `log.md` exist, and `okf-cli validate .` returns `OK`.

### 2. Validate
Run `okf-cli validate [path]`.

If it returns `ERR`, fix the listed issues first. Reach for `okf-cli tree index.md <depth>` only when you need to inspect relation shape in more detail.

*Completion*: `okf-cli validate [path]` returns `OK`.

### 3. Enrich
1. Add `title` and `description`.
2. Add a `# Schema` section with column tables.
3. Add `# Examples` and `# Citations`.
4. Weave cross-links into prose.

*Completion*: each concept carries metadata, a schema, and at least one citation or example, and its prose references its neighbours.

### 4. Convert sources
* **Notion** — map properties to frontmatter; strip UUID suffixes; convert links to relative.
* **Obsidian** — rewrite `[[wikilinks]]` to standard links; move inline `#tags` to frontmatter.
* **CSV** — one row per concept; column 1 = filename, other columns = frontmatter.

*Completion*: every source entity is a conformant concept (`type` present) with no residual wikilinks or UUIDs.

## Output format

Present a finished bundle as: (1) the directory tree, (2) each file in a fenced code block, (3) the line `"Bundle is OKF v0.1 conformant"`.
