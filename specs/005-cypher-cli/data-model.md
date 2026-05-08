# Data Model: Cypher CLI

**Feature**: 005-cypher-cli | **Date**: 2026-05-08

## Core Types

### `Severity`

Diagnostic severity level, matching LSP conventions and mirroring `gram/src/types.rs`.

```
Severity = Error | Warning | Information | Hint
```

Serialized as lowercase strings in JSON output.

---

### `AppliesTo`

Declares which parse tree(s) a rule's query targets.

```
AppliesTo = Structural | Contract | CrossReference
```

- `Structural` — query runs against the Cypher AST only
- `Contract` — query runs against the cypherdoc AST only
- `CrossReference` — both ASTs are provided; the Rust layer performs set-difference logic

---

### `Rule`

A loaded lint rule, compiled from a `.scm` file.

| Field | Type | Notes |
|-------|------|-------|
| `name` | `String` | e.g., `"UnlabelledNode"` — from `Rule:` header |
| `severity` | `Severity` | from `Severity:` header |
| `applies_to` | `AppliesTo` | from `Applies-to:` header |
| `message` | `String` | template from `Message:` header; `{name}` placeholder supported |
| `query` | `tree_sitter::Query` | compiled from the non-comment lines of the `.scm` file |

Rules where `applies_to = Contract` or `CrossReference` require `tree-sitter-cypherdoc` language.

---

### `Position`

Zero-indexed line and character (UTF-16 code unit) offset. Matches LSP and mirrors gram's type.

| Field | Type |
|-------|------|
| `line` | `u32` |
| `character` | `u32` |

---

### `Range`

Span covering a diagnostic location.

| Field | Type |
|-------|------|
| `start` | `Position` |
| `end` | `Position` |

---

### `Diagnostic`

A single finding emitted by a rule against a source file.

| Field | Type | Notes |
|-------|------|-------|
| `severity` | `Severity` | |
| `rule` | `String` | rule name (e.g., `"UnlabelledNode"`) |
| `message` | `String` | rendered message with any `{name}` placeholders filled |
| `range` | `Range` | source location |
| `code` | `Option<String>` | optional machine-readable code; omitted in JSON when absent |

---

### `FileResult`

All diagnostics for a single source file.

| Field | Type |
|-------|------|
| `path` | `String` |
| `diagnostics` | `Vec<Diagnostic>` |

---

### `LintResult`

Top-level JSON output envelope (mirrors gram's `CheckResult`).

| Field | Type | Notes |
|-------|------|-------|
| `schema_version` | `u32` | always `1` in this version |
| `tool` | `String` | e.g., `"cypher/0.1.0"` |
| `files` | `Vec<FileResult>` | one entry per checked file |

---

### `DocStatementPair`

Internal type used during cross-reference rule evaluation. Pairs a parsed cypherdoc tree with its adjacent Cypher statement node.

| Field | Type | Notes |
|-------|------|-------|
| `doc_tree` | `Option<tree_sitter::Tree>` | absent when statement has no preceding doc comment |
| `doc_source` | `Option<String>` | raw text of the doc comment (for ariadne source cache) |
| `doc_start_byte` | `usize` | byte offset of doc comment in the file (for position mapping) |
| `statement_node` | `tree_sitter::Node` | the `statement` node in the Cypher tree |

---

## Entity Relationships

```
LintResult
  └── FileResult (1..n)
        └── Diagnostic (0..n)
              ├── Severity
              ├── Range (start: Position, end: Position)
              └── rule: String  →  Rule.name

Rule
  ├── AppliesTo
  ├── Severity
  └── query: tree_sitter::Query

DocStatementPair  (runtime only, not serialized)
  ├── doc_tree: Option<cypherdoc Tree>
  └── statement_node: cypher Node
```

---

## State Transitions

The linter is stateless between files. For each file the lifecycle is:

1. Read source bytes
2. Parse with `tree-sitter-cypher` → Cypher tree
3. Walk `source_file` children → collect `DocStatementPair` list
4. For each pair with a `doc_comment`: parse doc text with `tree-sitter-cypherdoc` → `doc_tree`
5. For each `Rule`:
   - `Structural`: run `rule.query` against Cypher tree → emit `Diagnostic`s
   - `Contract`: run `rule.query` against each `doc_tree` → emit `Diagnostic`s
   - `CrossReference`: extract param sets from both trees → set-difference → emit `Diagnostic`s
6. Collect all `Diagnostic`s → build `FileResult`
