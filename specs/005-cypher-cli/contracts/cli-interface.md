# Contract: CLI Interface

**Feature**: 005-cypher-cli | **Date**: 2026-05-08

## Binary Name

`cypher`

## Sub-commands

### `cypher lint [OPTIONS] [PATHS...]`

Lint one or more `.cypher` files for structural, contract, and cross-reference issues.

#### Arguments

| Argument | Type | Description |
|----------|------|-------------|
| `[PATHS...]` | `PathBuf...` | Files or directories to lint. Omit to read from stdin. |

#### Options

| Flag | Type | Description |
|------|------|-------------|
| `-e, --expression <EXPR>` | `String` | Lint an inline Cypher expression instead of a file |
| `--json` | flag | Emit machine-readable JSON (see `json-output-schema.md`); conflicts with `--tree` |
| `--tree` | flag | Print the s-expression parse tree to stdout; conflicts with `--json` |
| `--strict` | flag | Treat warnings as errors (exit code 1 if any warning found) |
| `--rule <NAME>` | `String` (repeatable) | Run only the named rule(s); unknown name → exit 2 |
| `--rules-dir <PATH>` | `PathBuf` | Load additional `.scm` rule files from this directory |

#### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | No errors found (warnings ignored unless `--strict`) |
| `1` | One or more error-severity diagnostics found (or warnings under `--strict`) |
| `2` | Usage error, unknown rule name, unreadable file, or internal failure |

#### Stdin

When `PATHS` is empty and `-e` is not given, source is read from stdin. The path label in
output (and JSON) is `-`.

#### Directory Traversal

When a `PATHS` entry is a directory, `cypher lint` recursively finds all `*.cypher` files
within it using `walkdir`. A directory containing no `.cypher` files prints a note to stderr
and exits 0.

---

### `cypher <EXT-NAME> [ARGS...]`

Dispatch to an external `cypher-<EXT-NAME>` binary found on `PATH`. All remaining `ARGS` are
passed through unmodified. The process is replaced via `exec` on Unix.

**Error when not found**:
```
error: unknown sub-command '<EXT-NAME>'

No built-in sub-command '<EXT-NAME>' and no 'cypher-<EXT-NAME>' binary found on PATH.

Try 'cypher --help' for a list of built-in commands.
```
Exit code: `2`.

---

## Global Options

| Flag | Description |
|------|-------------|
| `--help` / `-h` | Print help |
| `--version` / `-V` | Print version (`cypher <version>`) |

---

## Standard Streams

- **stdout**: Machine-readable output only — `--json` diagnostics and `--tree` s-expressions
- **stderr**: Human-readable (pretty) diagnostics, notes, warnings about missing files, and error messages
- **stdin**: Source input when no paths given to `cypher lint`

---

## Environment

No required environment variables. The external dispatch reads `PATH` as provided by the shell.
