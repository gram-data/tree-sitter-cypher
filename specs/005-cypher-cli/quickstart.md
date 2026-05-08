# Quickstart: Cypher CLI

**Feature**: 005-cypher-cli | **Date**: 2026-05-08

## Prerequisites

- Rust stable toolchain (`rustup update stable`)
- The `tree-sitter-cypher` repo cloned locally (this repo)

## Build

```sh
# From the repository root
cargo build -p cypher-data

# Release build
cargo build -p cypher-data --release
```

The `cypher` binary lands at `target/debug/cypher` (or `target/release/cypher`).

## Run

```sh
# Lint a single file
./target/debug/cypher lint cypher/find_person.cypher

# Lint all .cypher files under a directory
./target/debug/cypher lint cypher/

# Lint from stdin
cat cypher/find_person.cypher | ./target/debug/cypher lint

# Inline expression
./target/debug/cypher lint -e 'MATCH (n) RETURN n'

# Machine-readable output
./target/debug/cypher lint --json cypher/

# Show parse tree (grammar debugging)
./target/debug/cypher lint --tree cypher/find_person.cypher

# Run a single rule
./target/debug/cypher lint --rule UnlabelledNode cypher/

# Strict mode (warnings become errors)
./target/debug/cypher lint --strict cypher/
```

## Test

```sh
# Run all tests (unit + integration)
cargo test -p cypher-data

# Run a single integration test by name
cargo test -p cypher-data --test lint_integration unlabelled_node

# Run with output visible
cargo test -p cypher-data -- --nocapture
```

## Add a New Rule

1. Create `tools/cypher/rules/<category>/<RuleName>.scm`:

   ```lisp
   ;; Rule: MyNewRule
   ;; Severity: Warning
   ;; Applies-to: structural
   ;; Message: Describe the problem and how to fix it.
   (some_node_type
     field: (something)) @capture
   ```

2. Add `include_str!("../rules/<category>/<RuleName>.scm")` to the built-in rule list in
   `src/rules.rs`.

3. Run `cargo test -p cypher-data` to verify the rule loads and compiles.

4. Add a fixture file to `tests/fixtures/` and a corresponding integration test.

No recompilation is needed for rules loaded via `--rules-dir`.

## Workspace Layout

```
tree-sitter-cypher/        ← repo root
├── Cargo.toml             ← workspace (includes tools/cypher)
├── grammar.js             ← Cypher grammar
├── tree-sitter-cypherdoc/ ← cypherdoc sub-grammar
└── tools/
    └── cypher/            ← this tool
        ├── Cargo.toml     ← package: cypher-data, bin: cypher
        ├── src/
        ├── rules/
        └── tests/
```
