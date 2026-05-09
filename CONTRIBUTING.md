# Contributing

## Prerequisites

- [Node.js](https://nodejs.org/) 22+ and npm
- [Rust](https://rustup.rs/) stable toolchain
- [tree-sitter CLI](https://tree-sitter.github.io/tree-sitter/creating-parsers): `npm install -g tree-sitter-cli`

## Development

```sh
# Install Node dependencies
npm install

# After any grammar.js change — regenerate the parser
tree-sitter generate

# Run all tests
make test

# Run the cypher CLI locally
cargo run -p cypher-data -- lint example.cypher
```

The openCypher BNF at `references/openCypher/grammar/openCypher.bnf` is the spec to implement. Each BNF production maps to a named rule in `grammar.js`.

### Cypherdoc sub-grammar

Changes to `tree-sitter-cypherdoc/grammar.js` need their own generate + test cycle:

```sh
cd tree-sitter-cypherdoc
tree-sitter generate
tree-sitter test
```

### Adding corpus tests

Corpus tests live in `test/corpus/*.txt`. Add a test alongside every new grammar rule:

```
================================================================================
MATCH with label
================================================================================

MATCH (n:Person) RETURN n

--------------------------------------------------------------------------------

(source_file
  (statement
    (match_clause ...)
    (return_clause ...)))
```

## Releasing

All three packages (`tree-sitter-cypher`, `tree-sitter-cypherdoc`, `cypher-data`) are released together from a single version tag. The publish workflow (`.github/workflows/publish.yml`) runs automatically on tag push.

### Required repository secrets

Before the first release, add these in **Settings → Secrets and variables → Actions**:

| Secret | Obtain from |
|--------|-------------|
| `NPM_TOKEN` | npmjs.com → Access Tokens (type: Automation) |
| `CARGO_REGISTRY_TOKEN` | crates.io → Account Settings → API Tokens |

### Release steps

1. Run the prepare-release script to version-align all packages:
   ```sh
   scripts/prepare-release.sh 0.2.0
   ```
   This updates `[workspace.package] version` in `Cargo.toml` (inherited by `cypher-data`),
   then calls `tree-sitter version` for each grammar to sync `package.json` and `[package] version`.

2. Commit and tag:
   ```sh
   git diff                          # verify changes
   git add Cargo.toml Cargo.lock package.json tree-sitter-cypherdoc/Cargo.toml tree-sitter-cypherdoc/package.json
   git commit -m "chore: release v0.2.0"
   git tag v0.2.0 && git push origin main --tags
   ```

The workflow will then run tests, publish to npm and crates.io, build cross-platform binaries, and create a GitHub Release with the binaries attached.
