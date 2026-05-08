# Research: Cypher CLI

**Feature**: 005-cypher-cli | **Date**: 2026-05-08

## Decision 1 — Workspace Layout

**Decision**: Add `tools/cypher` as a workspace member in the root `Cargo.toml`.

**Rationale**: The root `Cargo.toml` already defines a `[package]` for `tree-sitter-cypher`. Adding a `[workspace]` section (or upgrading to a workspace root) lets `tools/cypher` share the edition and dependency versions without a separate repository. The gram project uses exactly this layout (`tree-sitter-gram` workspace → `tools/gram` member).

**Alternatives considered**:
- Separate repository for the CLI — rejected; forces separate release cycle and makes path dependencies awkward.
- Flat crate at repo root — rejected; pollutes the grammar crate's public API surface with CLI dependencies.

---

## Decision 2 — Injected Grammar Handling in Rust

**Decision**: Parse the cypherdoc sub-grammar manually at the application layer — not via Tree-sitter's language injection API.

**Rationale**: Tree-sitter's injection mechanism is primarily a *editor-integration* feature (Neovim, Helix, etc. use it via `queries/injections.scm`). The Rust `tree-sitter` crate does not provide a built-in injected-tree API; editors implement injection themselves. For the linter:

1. Parse the full source with `tree-sitter-cypher` → get the Cypher tree.
2. Walk `source_file` children to find `doc_comment` nodes.
3. For each `doc_comment` node, slice `source[node.start_byte()..node.end_byte()]` and re-parse that slice with `tree-sitter-cypherdoc`.
4. Store the resulting `(doc_comment_node, cypherdoc_tree, statement_node)` triple for rule evaluation.

**Alternatives considered**:
- Using `queries/injections.scm` at runtime — not supported by the Rust tree-sitter crate without editor scaffolding.
- Parsing the entire file twice — unnecessary; the doc comment bytes are extracted from the first parse.

---

## Decision 3 — Rule Embedding Strategy

**Decision**: Embed built-in `.scm` rule files at compile time using `include_str!()`. Support a `--rules-dir <path>` flag for runtime-loaded additional rules.

**Rationale**: `include_str!()` is zero-cost and keeps the binary self-contained (no external rule files required for operation). A runtime flag is straightforward to add and satisfies the extensibility requirement without overcomplicating the initial implementation.

**Alternatives considered**:
- All rules loaded at runtime from `~/.cypher/rules/` — rejected for the built-ins; a missing or corrupted rule directory would silently disable rules.
- Generating a Rust enum from rule files via `build.rs` — more complex than `include_str!()` with no clear benefit for the use case.

---

## Decision 4 — Rule Header Parsing

**Decision**: Parse rule metadata from leading `;;`-prefixed comment lines in `.scm` files. A simple key-value parser reading lines until the first non-comment line is sufficient.

**Format** (established in `proposals/cypher-linter.md`):
```
;; Rule: UnlabelledNode
;; Severity: Warning
;; Applies-to: structural
;; Message: "MATCH (n)" causes a full node scan. Add a label, e.g., (n:Person).
(node_pattern ...)
```

**Rationale**: Trivial to parse with `str::lines()` + `strip_prefix(";; ")`. No external parser needed.

---

## Decision 5 — `MissingToolName` Detection

**Decision**: Implement `MissingToolName` as a Rust-layer check rather than a `.scm` query.

**Rationale**: The `proposals/cypher-linter.md` notes that a missing name produces a zero-width MISSING placeholder that is not queryable. The check is `node.child_by_field_name("name").map_or(true, |n| n.byte_range().is_empty())`. This is a single line of Rust; forcing it through the `.scm` machinery adds complexity without benefit.

---

## Decision 6 — ariadne vs miette

**Decision**: Use `ariadne 0.6` (same as the gram CLI).

**Rationale**: Consistency with gram. Users who see `gram check` output will immediately recognize `cypher lint` output. Both tools may be used in the same shell session or CI pipeline.

---

## Decision 7 — Extension / Skill Sub-commands

**Decision**: Defer `cypher extension` and `cypher skill` sub-commands to a future feature. The external dispatch mechanism (`cypher-<name>` PATH lookup) means both can be bootstrapped as external binaries without a built-in sub-command.

**Rationale**: The spec explicitly defers skill management. Extension management adds ~300 lines of code (install/list/remove + `~/.cypher/extensions.toml`) but no user scenario in this feature requires it. Ship the dispatch mechanism; the extension sub-command is a separate slice.

---

## Decision 8 — `--rule` Flag Syntax

**Decision**: Accept a single rule name per flag, repeatable: `--rule UnlabelledNode --rule UnboundedRelationship`. This is idiomatic clap derive with `Vec<String>`.

**Alternatives considered**:
- Comma-separated list — less idiomatic for clap; harder to shell-escape.
- Separate `--ignore-rule` flag — useful but deferred; not in the spec.
