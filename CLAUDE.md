# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan
at specs/005-cypher-cli/plan.md
<!-- SPECKIT END -->

## Project Overview

`tree-sitter-cypher` is a [Tree-sitter](https://tree-sitter.github.io/tree-sitter/) grammar for the Cypher graph query language (used by Neo4j). The grammar is currently a stub — the primary work is implementing Cypher syntax rules in `grammar.js`.

## Key Commands

```sh
# Regenerate parser from grammar.js (run after every grammar change)
tree-sitter generate

# Run grammar tests (corpus tests in test/corpus/)
tree-sitter test
# or
make test

# Run a single test by name filter
tree-sitter test -f "test name pattern"

# Run Node.js binding tests
npm test

# Interactive playground (builds wasm first)
npm start

# Build native C library
make

# Parse a file and show the syntax tree
tree-sitter parse example-file
```

## Cypherdoc Sub-grammar

The `tree-sitter-cypherdoc/` subdirectory is a self-contained Tree-sitter grammar for
structured `/** */` doc comments. It is injected into `doc_comment` nodes by this grammar
via `queries/injections.scm`.

```sh
# From tree-sitter-cypherdoc/ — regenerate after grammar.js changes
cd tree-sitter-cypherdoc && tree-sitter generate

# Run cypherdoc corpus tests
cd tree-sitter-cypherdoc && tree-sitter test

# Parse a .cypher file and see cypherdoc injection (from repo root)
tree-sitter parse cypher/find_person.cypher
```

## Architecture

**Source of truth**: `grammar.js` — Tree-sitter grammar using the `grammar()` DSL. All other generated files derive from this.

**Generation pipeline**:
1. `grammar.js` → `tree-sitter generate` → `src/grammar.json`
2. `src/grammar.json` → `tree-sitter generate` → `src/parser.c` + `src/node-types.json`
3. Native bindings build from `src/parser.c` via node-gyp / Cargo / etc.

**Bindings** (`bindings/`): Language-specific wrappers for C, Go, Java, Node.js, Python, Rust, Swift, and Zig. Mostly generated boilerplate — rarely need manual edits.

**Reference material** (`references/openCypher/`): The authoritative openCypher specification including:
- `grammar/openCypher.bnf` — BNF grammar spec to translate into Tree-sitter rules
- `tck/` — Technology Compatibility Kit: feature tests organized by Cypher feature area

**Tests**: Corpus tests live in `test/corpus/*.txt` using Tree-sitter's s-expression format. Add tests alongside grammar rules.

## Grammar Development Workflow

1. Edit `grammar.js` to add/modify rules
2. Run `tree-sitter generate` to regenerate the parser
3. Run `tree-sitter test` to verify all corpus tests pass
4. Add corpus tests in `test/corpus/` for new syntax
5. Use `tree-sitter parse example-file` to manually inspect parse trees

The openCypher BNF at `references/openCypher/grammar/openCypher.bnf` is the spec to implement. Tree-sitter grammar rules map closely to BNF productions — each `<rule-name>` in the BNF becomes a named rule in the `rules: {}` object.
