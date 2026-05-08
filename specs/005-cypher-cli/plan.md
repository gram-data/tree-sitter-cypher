# Implementation Plan: Cypher CLI

**Branch**: `005-cypher-cli` | **Date**: 2026-05-08 | **Spec**: [spec.md](spec.md)  
**Input**: Feature specification from `specs/005-cypher-cli/spec.md`

## Summary

Build a `cypher` CLI tool (package `cypher-data`) that lints `.cypher` files using both the
`tree-sitter-cypher` grammar and the `tree-sitter-cypherdoc` injected sub-grammar.
The tool mirrors the `gram` CLI (`../tree-sitter-gram/tools/gram/`) in structure, crate layout,
and UX conventions — same clap/ariadne/walkdir stack, same external-dispatch pattern — so that
the two tools behave consistently when used together.

## Technical Context

**Language/Version**: Rust stable (edition 2021), matching the gram workspace  
**Primary Dependencies**: clap 4.5 (derive), ariadne 0.6, walkdir 2, serde/serde_json 1, tree-sitter 0.25+, directories 5  
**Grammar Dependencies**: tree-sitter-cypher (path `../..`), tree-sitter-cypherdoc (path `../../tree-sitter-cypherdoc`)  
**Storage**: N/A (stateless file analysis; extension registry in `~/.cypher/extensions.toml` mirrors gram pattern)  
**Testing**: `cargo test` — unit tests inline, integration tests via `assert_cmd` + `predicates` + fixture `.cypher` files  
**Target Platform**: Linux, macOS, Windows — x86_64 and ARM64 (matching gram distribution targets)  
**Project Type**: CLI binary  
**Performance Goals**: Single file lint completes in < 500 ms including grammar loading  
**Constraints**: Single statically-linked binary; no external database; no network access for core lint  
**Scale/Scope**: Designed for files up to ~10 000 lines; batch mode for directories of hundreds of files

## Constitution Check

*GATE: These gates guard `grammar.js` changes. The CLI does not modify the grammar, so gates I–III are N/A for the CLI implementation itself. The CLI must not bypass grammar validation.*

| Gate | Status | Notes |
|------|--------|-------|
| **Fidelity gate** | N/A | CLI adds no grammar rules; grammar.js is unchanged |
| **Dual-coverage gate** | N/A | CLI adds no corpus tests; grammar test suite is unchanged |
| **TCK gate** | N/A | CLI depends on the grammar's TCK compliance; does not affect it |

## Project Structure

### Documentation (this feature)

```text
specs/005-cypher-cli/
├── plan.md              ← this file
├── research.md          ← Phase 0 output
├── data-model.md        ← Phase 1 output
├── quickstart.md        ← Phase 1 output
├── contracts/           ← Phase 1 output
│   ├── cli-interface.md
│   └── json-output-schema.md
└── tasks.md             ← Phase 2 output (/speckit-tasks — NOT created here)
```

### Source Code (repository root)

The new tool lives under `tools/cypher/`, mirroring `../tree-sitter-gram/tools/gram/`.
The root `Cargo.toml` gains a workspace entry; `tools/cypher/Cargo.toml` declares the
`cypher-data` package.

```text
tools/cypher/
├── Cargo.toml                   # package: cypher-data, bin: cypher
├── src/
│   ├── main.rs                  # Cli struct, Commands enum, main()
│   ├── lint.rs                  # LintArgs, run() — structural + contract + cross-ref
│   ├── dispatch.rs              # external cypher-<name> binary dispatch
│   ├── rules.rs                 # .scm rule loading, header parsing, Rule struct
│   └── types.rs                 # Diagnostic, Severity, Range, Position, FileResult, LintResult
├── rules/
│   ├── structural/
│   │   ├── unlabelled_node.scm
│   │   └── unbounded_relationship.scm
│   ├── contract/
│   │   └── optional_param_missing_default.scm
│   └── cross_reference/
│       ├── undocumented_parameter.scm
│       └── unused_parameter.scm
└── tests/
    ├── lint_integration.rs
    └── fixtures/
        ├── clean.cypher
        ├── unlabelled_node.cypher
        ├── unbounded_relationship.cypher
        ├── unused_param.cypher
        └── undocumented_param.cypher
```

**Structure Decision**: Single project under `tools/cypher/`, added to the repo's Cargo workspace. Identical layout to `tools/gram/` in the gram repo so contributors familiar with one can navigate the other.

## Complexity Tracking

*No constitution violations. Not applicable.*

---

## Phase 0: Research

*All NEEDS CLARIFICATION items resolved below — no unknowns remain after examining the gram CLI source and tree-sitter-cypherdoc design.*

See [research.md](research.md) for full findings.

---

## Phase 1: Design

Artifacts generated in this phase:

- [data-model.md](data-model.md) — core types
- [contracts/cli-interface.md](contracts/cli-interface.md) — command-line API contract
- [contracts/json-output-schema.md](contracts/json-output-schema.md) — `--json` output schema
- [quickstart.md](quickstart.md) — build, run, test guide

---

## Implementation Slices

Ordered by dependency; each slice is independently shippable.

### Slice 1 — Scaffold & Single-File Structural Lint

**Goal**: `cypher lint file.cypher` reports unlabelled-node and unbounded-relationship warnings using the Cypher AST only.

**Deliverables**:
- `tools/cypher/Cargo.toml` (package `cypher-data`, bin `cypher`)
- Root `Cargo.toml` workspace entry
- `src/main.rs`, `src/dispatch.rs`, `src/types.rs`
- `src/lint.rs` — stdin + file path input, `--json`, `--strict`, `--tree` flags
- `src/rules.rs` — `.scm` header parser, `Rule` struct, `include_str!` embedding
- `rules/structural/unlabelled_node.scm`
- `rules/structural/unbounded_relationship.scm`
- Integration tests: `clean.cypher` (exit 0), `unlabelled_node.cypher` (exit 1), `unbounded_relationship.cypher` (exit 1)

**Done when**: `cargo test` passes; `cypher lint` on all fixture files produces correct output and exit codes.

---

### Slice 2 — Directory / Batch Mode

**Goal**: `cypher lint <dir>` recursively finds and checks all `.cypher` files.

**Deliverables**:
- `walkdir` integration in `lint.rs`
- Integration test: directory with mixed-result files → correct per-file output, aggregate exit code

**Done when**: `cargo test` passes; batch lint exits non-zero when any file has errors.

---

### Slice 3 — Contract Rules (cypherdoc)

**Goal**: `cypher lint file.cypher` also applies contract rules from cypherdoc doc comments.

**Deliverables**:
- `rules/contract/optional_param_missing_default.scm`
- `lint.rs` — doc-comment node extraction, separate cypherdoc parse, `MissingToolName` Rust-layer check
- `rules/contract/missing_tool_name.scm` (or Rust-layer check only — see research.md)
- Fixture `optional_param_error.cypher`, `missing_tool_name.cypher`
- Integration tests for both contract rules

**Done when**: Contract diagnostics appear with correct line/column from the cypherdoc parse tree.

---

### Slice 4 — Cross-Reference Rules

**Goal**: Correlate `@param` declarations in cypherdoc with `$parameter` usages in the Cypher body.

**Deliverables**:
- `rules/cross_reference/undocumented_parameter.scm`
- `rules/cross_reference/unused_parameter.scm`
- `lint.rs` — `(doc_comment, statement)` pair walking, set-difference logic
- Fixtures `unused_param.cypher`, `undocumented_param.cypher`
- Integration tests for both cross-reference rules

**Done when**: All five rule categories from `proposals/cypher-linter.md` pass integration tests.

---

### Slice 5 — External Dispatch

**Goal**: `cypher <name> [args]` exec's `cypher-<name>` from PATH.

**Deliverables**:
- `src/dispatch.rs` (mirrors `gram/src/dispatch.rs` exactly, substituting `cypher-` prefix)
- Integration test using a dummy `cypher-hello` fixture script

**Done when**: External dispatch works on Linux/macOS; error message is clear when binary not found.

---

### Slice 6 — Rule Filtering & `--rule` Flag

**Goal**: `cypher lint --rule UnlabelledNode file.cypher` runs only the named rule.

**Deliverables**:
- `--rule <name>` flag in `LintArgs`
- Filter logic in `lint.rs` before query execution
- Integration test: `--rule` with valid name produces only matching diagnostics; invalid name exits 2

**Done when**: `cargo test` passes; `--rule` narrows output correctly.

---

### Slice 7 — Distribution & CI

**Goal**: Binary is cross-compiled and released via cargo-dist (matching gram targets).

**Deliverables**:
- `[package.metadata.dist]` targets in `tools/cypher/Cargo.toml`
- GitHub Actions workflow (or extension of existing workflow) for CI lint + test
- `README.md` for `tools/cypher/` with install + usage instructions

**Done when**: `cargo build --release` succeeds on all target platforms in CI.
