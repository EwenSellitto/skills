# Open Knowledge Format (OKF)

OKF = directory of markdown files + YAML frontmatter. Represent knowledge metadata + context. Human readable, agent parseable. No central authority. Read via `cat`. Ship via `git clone`.

---

## 1. Motivation

### Goals

* Human readable without tools.
* Agent parseable without custom SDKs.
* Git diffable.
* Portable across tools + time.
* Enrichment agents write → consumption agents read.

### Non-goals

* No fixed taxonomy.
* No prescriptive storage/query infra.
* No replacing domain schemas (Avro, Protobuf, OpenAPI). OKF references them.

---

## 2. Terminology

* **Knowledge Bundle** — Hierarchical directory tree. Unit of distribution.
* **Concept** — One markdown file. Unit of knowledge. Describes physical asset or abstract idea.
* **Concept ID** — File path minus `.md` suffix (e.g., `tables/users.md` → `tables/users`).
* **Frontmatter** — YAML metadata block top of file between `---`.
* **Body** — Markdown content below frontmatter.
* **Link** — Markdown link between concepts. Assert untyped relationship edge.
* **Citation** — Link to external source backing body claim.

---

## 3. Bundle Structure

Arbitrary tree. Directory structure independent of domain.

```
path/to/bundle/
├── index.md                      # Optional. Directory listing.
├── log.md                        # Optional. Update history.
├── <concept>.md                  # Root concept.
└── <subdirectory>/               # Groups.
    ├── index.md
    └── <concept>.md

```

Ship via: Git repo, tar/zip archive, or repository subdirectory.

### 3.1 Reserved Filenames

| Filename | Purpose |
| --- | --- |
| `index.md` | Directory listing (§6) |
| `log.md` | Update history (§7) |

Do not use reserved names for concept documents.

---

## 4. Concept Documents

UTF-8 markdown files. Two parts: YAML frontmatter + markdown body.

### 4.1 Frontmatter

```yaml
---
type: <Type name>                  # REQUIRED
title: <Optional display name>
description: <Optional summary>
resource: <Optional canonical URI>
tags: [<tag>, <tag>, …]            # Optional
timestamp: <ISO 8601 datetime>     # Optional
# Producer-defined custom keys allowed
---

```

* `type` — Filter/route target. Unknown types → treat as generic.
* `title` — Display name. Fallback to filename if missing.
* `description` — Single sentence summary for snippets/previews.
* `resource` — Canonical URI for underlying asset. Omit if abstract idea.
* Unknown keys — Consumers must preserve, not reject.

### 4.2 Body

Favor structural markdown (headings, lists, tables, code blocks).
Conventional headings:

* `# Schema` — Columns/fields descriptors.
* `# Examples` — Code/usage blocks.
* `# Citations` — Sourced items (§8).

### 4.3 Example: Resource Concept

```markdown
---
type: BigQuery Table
title: Customer Orders
description: One row per completed customer order across all channels.
resource: https://console.cloud.google.com/bigquery?p=acme&d=sales&t=orders
tags: [sales, orders, revenue]
timestamp: 2026-05-28T14:30:00Z
---

# Schema

| Column        | Type      | Description                              |
|---------------|-----------|------------------------------------------|
| `order_id`    | STRING    | Globally unique order identifier.        |
| `customer_id` | STRING    | Foreign key into [customers](/tables/customers.md). |
| `total_usd`   | NUMERIC   | Order total in US dollars.               |
| `placed_at`   | TIMESTAMP | When the customer submitted the order.   |

# Joins

Joined with [customers](/tables/customers.md) on `customer_id`.

# Citations

[1] [BigQuery table schema](https://console.cloud.google.com/bigquery?p=acme&d=sales&t=orders)

```

### 4.4 Example: Abstract Concept

```markdown
---
type: Playbook
title: Incident response — data freshness alert
description: Steps to triage a freshness alert on the orders pipeline.
tags: [oncall, incident]
timestamp: 2026-04-12T09:00:00Z
---

# Trigger

A freshness alert fires when `orders` lags more than 30 minutes behind
its expected SLA. See the [orders table](/tables/orders.md).

# Steps

1. Check the [ingestion job dashboard](https://example.com/dash).
2. …

```

---

## 5. Cross-linking

### 5.1 Absolute Links

Begin with `/`. Bundle root relative. Stable if file moves within subdir. Recommended.

```markdown
[text](/tables/customers.md)

```

### 5.2 Relative Links

Standard relative paths.

```markdown
[text](./other.md)

```

### 5.3 Semantics

Links = directed edges. Prose gives meaning. Broken links allowed (means unwritten knowledge).

---

## 6. Index Files

`index.md` enables progressive disclosure. No frontmatter. Lists content via sections:

```markdown
# Group Heading

* [Title](relative-url) - description matching frontmatter

```

Auto-generation allowed.

---

## 7. Log Files

`log.md` tracks change history scope. Flat list. Newest first. Headings use ISO 8601 `YYYY-MM-DD`.

```markdown
# Directory Update Log

## 2026-05-22
* **Update**: Added new BigQuery table reference for [Customer Metrics](/tables/customer-metrics.md).

```

---

## 8. Citations

Bottom of document under `# Citations`. Numbered list.

```markdown
# Citations

[1] [Public dataset announcement](https://cloud.google.com/...)

```

---

## 9. Conformance

Bundle conformant if:

1. All non-reserved `.md` files have parseable YAML frontmatter.
2. All frontmatter has non-empty `type`.
3. Reserved files follow §6 and §7 rules if present.

Do not reject bundle for: missing optional fields, unknown types/keys, broken links, missing indices. Permissive consumption model intentional.

---

## 10. Alignment & Versioning

Similar to LLM wikis, Obsidian, metadata-as-code. OKF adds spec for interoperability.

Version format: `<major>.<minor>`. Target version declared via `okf_version: "0.1"` in root `index.md` frontmatter (only exception to index frontmatter ban). Unknown versions → best-effort consumption.

---

## Appendix A — Minimal Bundle

```
my_bundle/
├── index.md
├── datasets/
│   ├── index.md
│   └── sales.md
└── tables/
    ├── index.md
    ├── orders.md
    └── customers.md

```

`datasets/sales.md`:

```markdown
---
type: BigQuery Dataset
title: Sales
description: All sales-related tables for the retail business.
resource: https://console.cloud.google.com/bigquery?p=acme&d=sales
tags: [sales]
timestamp: 2026-05-28T00:00:00Z
---

The sales dataset contains transactional tables, including
[orders](/tables/orders.md) and [customers](/tables/customers.md).

```

`tables/orders.md`:

```markdown
---
type: BigQuery Table
title: Orders
description: One row per completed customer order.
resource: https://console.cloud.google.com/bigquery?p=acme&d=sales&t=orders
tags: [sales, orders]
timestamp: 2026-05-28T00:00:00Z
---

# Schema

| Column        | Type      | Description                  |
|---------------|-----------|------------------------------|
| `order_id`    | STRING    | Unique order identifier.     |
| `customer_id` | STRING    | FK to [customers](/tables/customers.md). |
| `total_usd`   | NUMERIC   | Order total in USD.          |

Part of the [sales dataset](/datasets/sales.md).

```
