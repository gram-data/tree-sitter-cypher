# Quickstart: Lint Markdown Cypher Snippets

**Feature**: 006-lint-markdown-cypher | **Date**: 2026-05-10

## Prerequisites

- Feature 005-cypher-cli must be complete (`tools/cypher/` builds and `cargo test` passes)
- Rust stable toolchain installed

## Build

```sh
# From repo root — builds the whole workspace including tools/cypher
cargo build

# Release build
cargo build --release
```

## Run the New Behavior

```sh
# Lint a single markdown file
cargo run --bin cypher -- lint README.md

# Lint a directory (checks both .cypher and .md files)
cargo run --bin cypher -- lint docs/

# Skip markdown files
cargo run --bin cypher -- lint docs/ --no-markdown

# JSON output for CI integration
cargo run --bin cypher -- lint --json README.md
```

## Run Tests

```sh
# All tests (unit + integration)
cargo test --manifest-path tools/cypher/Cargo.toml

# Only the new markdown unit tests
cargo test --manifest-path tools/cypher/Cargo.toml markdown

# Only markdown integration tests
cargo test --manifest-path tools/cypher/Cargo.toml lint_markdown
```

## Adding a Test Fixture

1. Create `tools/cypher/tests/fixtures/my_fixture.md` with a ` ```cypher ` block
2. Add an integration test in `tools/cypher/tests/lint_integration.rs`:

```rust
#[test]
fn lint_markdown_my_fixture() {
    Command::cargo_bin("cypher")
        .unwrap()
        .args(["lint", "tests/fixtures/my_fixture.md"])
        .current_dir(manifest_dir())
        .assert()
        .failure()  // or .success()
        .stderr(predicate::str::contains("ExpectedRuleName"));
}
```

## Verifying Line Numbers

To confirm that a diagnostic points to the correct line in a markdown file:

```sh
# Count to the expected line manually
grep -n "MATCH (n)" README.md

# Compare with cypher lint output
cargo run --bin cypher -- lint README.md 2>&1 | grep "UnlabelledNode"
```

Both should report the same line number.
