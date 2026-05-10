# Implementation Plan: Lint Markdown Cypher Snippets

**Branch**: `006-lint-markdown-cypher` | **Date**: 2026-05-10 | **Spec**: [spec.md](spec.md)  
**Input**: Feature specification from `specs/006-lint-markdown-cypher/spec.md`

## Summary

Extend the existing `cypher lint` command (introduced in 005-cypher-cli) to accept `.md` files
as input. Cypher code blocks fenced with ` ```cypher ` are extracted using a line-based state
machine, linted using the same rules applied to `.cypher` files, and diagnostics are reported
with line numbers relative to the original markdown file. No new external dependencies are
required — the feature is a purely additive extension to `tools/cypher/`.

## Technical Context

**Language/Version**: Rust stable (edition 2021), matching the cypher-data workspace  
**Primary Dependencies**: Same as 005-cypher-cli (clap 4.5, ariadne 0.6, walkdir 2, serde/serde_json 1, tree-sitter 0.25+). No new dependencies added.  
**Storage**: N/A (stateless file analysis; no new state)  
**Testing**: `cargo test` — unit tests for snippet extraction, integration tests via `assert_cmd` + fixture `.md` files  
**Target Platform**: Linux, macOS, Windows — x86_64 and ARM64 (same as 005)  
**Project Type**: Additive extension to existing CLI binary (`tools/cypher/`)  
**Performance Goals**: Linting a markdown file with 10 Cypher fenced blocks < 500 ms (same as single `.cypher` file target)  
**Constraints**: No new binary; no new crate dependencies; must not regress existing `.cypher` lint behavior  
**Scale/Scope**: Files with tens to hundreds of snippets; batch mode across `docs/` directories

## Constitution Check

*GATE: These gates guard `grammar.js` changes. This feature modifies only the CLI tool and adds
no grammar rules. All three gates are N/A — identical reasoning to 005-cypher-cli.*

| Gate | Status | Notes |
|------|--------|-------|
| **Fidelity gate** | N/A | No grammar rules added; `grammar.js` is unchanged |
| **Dual-coverage gate** | N/A | No corpus tests added; grammar test suite is unchanged |
| **TCK gate** | N/A | CLI depends on grammar TCK compliance; does not affect it |

## Project Structure

### Documentation (this feature)

```text
specs/006-lint-markdown-cypher/
├── plan.md              ← this file
├── research.md          ← Phase 0 output
├── data-model.md        ← Phase 1 output
├── quickstart.md        ← Phase 1 output
├── contracts/           ← Phase 1 output
│   └── cli-interface.md
└── tasks.md             ← Phase 2 output (/speckit-tasks — NOT created here)
```

### Source Code (repository root)

Extension to the existing `tools/cypher/` package. One new source file; several files extended.

```text
tools/cypher/
├── src/
│   ├── main.rs          (unchanged)
│   ├── lint.rs          (extend: --no-markdown flag, .md file routing, offset merging)
│   ├── markdown.rs      (NEW: snippet extraction state machine)
│   ├── dispatch.rs      (unchanged)
│   ├── rules.rs         (unchanged)
│   └── types.rs         (unchanged)
└── tests/
    ├── lint_integration.rs          (extend: markdown test cases)
    └── fixtures/
        ├── markdown_clean.md        (NEW)
        ├── markdown_unlabelled.md   (NEW)
        ├── markdown_multi_snippet.md (NEW)
        ├── markdown_empty_snippet.md (NEW)
        └── markdown_no_fence.md     (NEW)
```

**Structure Decision**: All changes are confined to `tools/cypher/`. No new crate, no workspace
change. The new `markdown.rs` module is the only new file; everything else is extension of
existing code.

## Complexity Tracking

*No constitution violations. Not applicable.*

---

## Phase 0: Research

See [research.md](research.md) for full findings.

---

## Phase 1: Design

Artifacts generated in this phase:

- [data-model.md](data-model.md) — new types and extensions to existing types
- [contracts/cli-interface.md](contracts/cli-interface.md) — updated CLI contract including `.md` support and `--no-markdown`
- [quickstart.md](quickstart.md) — build and test guide for this feature

---

## Implementation Slices

Ordered by dependency; each slice is independently testable.

### Slice 1 — Snippet Extraction Module

**Goal**: `markdown::extract_cypher_snippets(source: &str) -> Vec<CypherSnippet>` correctly
identifies all ` ```cypher ` blocks and records their content and starting line.

**Deliverables**:
- `tools/cypher/src/markdown.rs` — `CypherSnippet` struct, `extract_cypher_snippets()` function
- Unit tests inside `markdown.rs` covering: single snippet, multiple snippets, no snippets,
  empty snippet, unclosed fence, mixed-case ` ```Cypher `, language tag with trailing text
  (` ```cypher ` with annotation), language variants that should be skipped (` ```cql `)

**Done when**: `cargo test` passes for all unit tests in `markdown.rs`.

---

### Slice 2 — Lint Integration for `.md` Files

**Goal**: `cypher lint README.md` lints all Cypher snippets in the file and reports diagnostics
with line numbers relative to the markdown file.

**Deliverables**:
- `lint.rs` — extend `run()` to detect `.md` files by extension; add `lint_markdown_file()` helper
  that: reads full markdown source, calls `extract_cypher_snippets()`, calls `analyze()` per
  snippet, offsets all diagnostic line numbers by `snippet.start_line`, merges all snippet
  diagnostics into one `SourceResult` per markdown file with `source = full_markdown_text`
- `LintArgs` — no new flag yet (that's Slice 4)
- Integration tests: `markdown_clean.md` → exit 0, `markdown_unlabelled.md` → exit 1 with
  correct line number, `markdown_multi_snippet.md` → diagnostics from each block correctly
  attributed

**Done when**: `cargo test` passes; `cypher lint markdown_unlabelled.md` points to the correct
line inside the markdown file.

---

### Slice 3 — Directory Walk Extended to `.md`

**Goal**: `cypher lint docs/` discovers and checks `.md` files alongside `.cypher` files.

**Deliverables**:
- `lint.rs` — extend `WalkDir` filter to include `extension == "md"` in addition to `"cypher"`
- Integration test: directory with mixed `.cypher` and `.md` files → all checked, aggregate
  exit code reflects errors from either type

**Done when**: `cargo test` passes; batch lint on a mixed directory reports both file types.

---

### Slice 4 — `--no-markdown` Flag

**Goal**: `cypher lint --no-markdown` skips `.md` files entirely.

**Deliverables**:
- `LintArgs` — add `--no-markdown: bool` flag
- `lint.rs` — skip `.md` files (directory walk and explicit paths) when flag is set; print
  warning when explicit `.md` path is skipped under `--no-markdown`
- Integration tests: `--no-markdown` with directory → only `.cypher` checked; `--no-markdown`
  with explicit `.md` path → skipped with warning, exit 0

**Done when**: `cargo test` passes; `--no-markdown` suppresses all markdown processing.
