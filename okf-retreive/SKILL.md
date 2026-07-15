---
name: okf-retreive
description: Read-only OKF retrieval and parsing reference.
disable-model-invocation: true
---

# Retrieve OKF

Parse OKF **cheap first**: extract only the contract the spec guarantees, then deepen only if the task needs it.

## Steps

### 1. Bound the bundle

Identify the bundle root, then enumerate markdown files under it.

Treat files as three classes:

* `index.md`
* `log.md`
* concept files: every other `.md` file

Completion criterion: every markdown file in the bundle is classified once.

### 2. Parse the contract first

For each concept file, split the file into:

* YAML frontmatter at the top between `---`
* markdown body below it

Parse only what OKF requires for conformant consumption:

* frontmatter must be parseable YAML
* `type` must exist and be non-empty
* unknown frontmatter keys are preserved, not interpreted away

For reserved files:

* `index.md` has no frontmatter, except root `index.md` may carry `okf_version: "0.1"`
* `log.md` has no frontmatter

Completion criterion: every concept file is reduced to `id`, frontmatter map, and body; every reserved file is recognized under the reserved-file rules.

### 3. Build the retrieval record

Derive a stable record per concept:

* `id`: file path minus `.md`
* `path`
* `type`
* `title`: frontmatter `title`, else filename
* `description`
* `resource`
* `tags`
* `timestamp`
* `body`

Then extract cheap body structure without semantic guesswork:

* headings
* markdown links
* `# Citations` items
* `# Schema` tables
* `# Examples` code blocks

Completion criterion: every concept has one record with metadata plus raw structural extracts.

### 4. Resolve links predictably

Resolve markdown links as untyped directed edges.

If the task is to inspect relations from one concept outward, use `okf-cli tree` before doing a wider manual link walk.

Use these rules:

* absolute links begin with `/` and are bundle-root relative
* relative links resolve from the current file
* broken links stay in the graph as broken edges
* remote links are external references, not concepts
* non-`.md` links are leaves, not concept parses

Do not infer relationship meaning from link text alone; OKF leaves semantics in the surrounding prose.

Completion criterion: every markdown link is classified as internal concept edge, broken internal edge, external link, or non-markdown leaf.

### 5. Stop at the cheapest sufficient depth

Default retrieval depth:

* parse metadata for every concept
* parse structural sections only when requested or obviously relevant
* walk links outward from the user's starting concept instead of loading the whole bundle body-first

Reach for deeper parsing only when the task depends on it:

* `# Schema` for fields and columns
* `# Examples` for usage
* `# Citations` for source backing
* linked concepts for local context expansion

Completion criterion: the returned parse includes enough structure for the task, without upgrading the whole bundle to deep parsing by default.

## Reference

### Efficient retrieval order

Use this order every time:

1. find the bundle root
2. list `.md` files
3. classify reserved vs concept files
4. parse frontmatter and keep body raw
5. extract links and conventional sections
6. deepen only around the concepts the task actually touches

### What to treat as authoritative

The stable parse contract is small:

* markdown file tree
* YAML frontmatter
* required `type`
* reserved filenames
* standard markdown links

Everything else is permissive:

* unknown keys are allowed
* unknown `type` values are allowed
* broken links are allowed
* missing optional fields are allowed
* missing index files are allowed

### Parsing posture

Parse OKF as a document graph, not as a strict schema system.

That means:

* preserve producer-defined keys
* keep bodies as markdown, not rewritten data models
* extract structure before interpretation
* treat prose as meaning and links as edges

### CLI

`okf-cli tree` can retrieve relation shape quickly, and `okf-cli metadata` can show frontmatter plus relations:

```bash
okf-cli tree [--show-non-md] <start_file.md> [max_depth]
okf-cli metadata [--show-non-md] <start_file.md> [max_depth]
okf-cli tag [--dir <start_dir>] <tag> [<tag> ...]
okf-cli tag [--dir <start_dir>]
```

Use it to:

* walk concept-to-concept relations from an entry file
* spot broken internal links as `!`
* spot cycles as `~`
* show non-markdown and remote leaves when `--show-non-md` is useful
* filter directory entries by one or more frontmatter tags with `okf-cli tag`
* list discovered tags with `okf-cli tag` and no tag arguments

### Context pointer

For the full format rules and examples, read `./SPEC.md`.
