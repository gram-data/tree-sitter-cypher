# Contract: CLI Interface (006 — Markdown Cypher Lint)

**Feature**: 006-lint-markdown-cypher | **Date**: 2026-05-10  
**Extends**: `specs/005-cypher-cli/contracts/cli-interface.md`

This document records only the additions and changes to the CLI contract introduced by feature
006. All clauses from 005 remain in effect unless explicitly overridden here.

---

## Changes to `cypher lint [OPTIONS] [PATHS...]`

### Extended `[PATHS...]` Argument

In addition to `.cypher` files and directories, `[PATHS...]` now accepts `.md` files.

| Path type | Behavior |
|-----------|----------|
| `*.cypher` | Unchanged from 005 |
| `*.md` | Cypher fenced blocks extracted and linted; diagnostics reported with line numbers in the `.md` file |
| directory | Recursively finds both `*.cypher` **and** `*.md` files (unless `--no-markdown` is set) |

### New Option: `--no-markdown`

| Flag | Type | Description |
|------|------|-------------|
| `--no-markdown` | flag | Skip all `.md` files during directory traversal and explicit-path processing |

**Behavior when `--no-markdown` is set with an explicit `.md` path**:
```
note: README.md: skipped (--no-markdown)
```
Exit code: `0` (no error; the flag is intentional).

### Directory Traversal (updated)

When a `PATHS` entry is a directory, `cypher lint` recursively finds all `*.cypher` and `*.md`
files using `walkdir`, unless `--no-markdown` is passed (in which case only `*.cypher` files
are found). A directory containing no eligible files prints a note to stderr and exits 0.

---

## Markdown Snippet Extraction

A Cypher fenced block is any code fence whose language tag (the first whitespace-delimited
token after the opening backticks) equals `cypher` (case-insensitive).

**Recognized** (all map to Cypher lint):
```
```cypher
```Cypher
```CYPHER
```cypher title="Example"
```

**Not recognized** (silently skipped):
```
```cql
```cypher-shell
```cypherdoc
```

Empty snippets (only whitespace between fences) are silently skipped.

An unclosed fence (opening ` ``` ` with no matching closing ` ``` `) lints the remaining file
content as the snippet and emits a note to stderr.

---

## Diagnostic Output for Markdown Files

Diagnostics for `.md` files use the same format as `.cypher` files:

- **Human-readable**: `README.md:42:5: warning[UnlabelledNode]: ...`  
  Line 42 is the absolute line in the markdown file where the issue occurs.
- **JSON**: Path is the `.md` file; `range.start.line` is the absolute line in the markdown file.

The `--json` schema version remains `1`. No new required fields are added. The consumer can
identify the file type from the path extension.

---

## Exit Codes (unchanged from 005)

| Code | Meaning |
|------|---------|
| `0` | No errors found (warnings ignored unless `--strict`) |
| `1` | One or more error-severity diagnostics found (or warnings under `--strict`) |
| `2` | Usage error, unknown rule name, unreadable file, or internal failure |
