# cypher

A CLI tool for linting `.cypher` files using the [`tree-sitter-cypher`](../../README.md) grammar and the `tree-sitter-cypherdoc` sub-grammar.

## Install

```sh
# From the repository root
cargo build -p cypher-data --release
# Binary at target/release/cypher — add to PATH as desired
```

## Usage

```sh
# Lint a single file
cypher lint query.cypher

# Lint all .cypher files under a directory
cypher lint src/queries/

# Lint from stdin
echo 'MATCH (n) RETURN n' | cypher lint

# Inline expression
cypher lint -e 'MATCH (n:Person)-[*]-(b:Person) RETURN n, b'

# Machine-readable JSON output
cypher lint --json src/queries/

# Treat warnings as errors (useful in CI)
cypher lint --strict src/queries/

# Show the parse tree (grammar debugging)
cypher lint --tree query.cypher

# Run only specific rules
cypher lint --rule UnlabelledNode --rule UnusedParameter src/queries/

# Load additional rules from a directory (no recompile required)
cypher lint --rules-dir my-rules/ src/queries/
```

## Built-in Rules

### Structural (Cypher AST)

| Rule | Severity | Description |
|------|----------|-------------|
| `UnlabelledNode` | Warning | Node pattern `(n)` without a label causes a full node scan |
| `UnboundedRelationship` | Error | Variable-length `[*]` without an upper limit can hang the database |

### Contract (cypherdoc `/** */` comments)

| Rule | Severity | Description |
|------|----------|-------------|
| `OptionalParamMissingDefault` | Error | Optional `@param` written as bare `[name]` instead of `[name="default"]` |
| `MissingToolName` | Warning | Doc comment has no tool name on the first line |

### Cross-Reference (both grammars)

| Rule | Severity | Description |
|------|----------|-------------|
| `UnusedParameter` | Warning | `@param` declared in doc comment but `$name` never appears in the query |
| `UndocumentedParameter` | Warning | `$param` used in query but not declared as `@param` in the doc comment |

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | No errors (warnings are informational unless `--strict`) |
| `1` | One or more error-severity diagnostics found |
| `2` | Usage error, unknown rule name, or internal failure |

## JSON Output Example

```json
{
  "schema_version": 1,
  "tool": "cypher/0.1.0",
  "files": [
    {
      "path": "find_person.cypher",
      "diagnostics": [
        {
          "severity": "warning",
          "rule": "UnusedParameter",
          "message": "@param \"label\" is declared but $label never appears in the query.",
          "range": {
            "start": { "line": 2, "character": 12 },
            "end":   { "line": 2, "character": 17 }
          }
        }
      ]
    }
  ]
}
```

## Writing Custom Rules

Create a `.scm` file with a header block and a Tree-sitter query:

```lisp
;; Rule: CartesianProduct
;; Severity: Warning
;; Applies-to: structural
;; Message: Multiple MATCH clauses without a connecting relationship produce a Cartesian product.
(match_clause) @match
```

Then pass the directory containing it:

```sh
cypher lint --rules-dir ./my-rules src/queries/
```

## Extending with External Sub-commands

Any binary named `cypher-<name>` on `PATH` is automatically available as `cypher <name>`:

```sh
# If cypher-format is on PATH:
cypher format src/queries/
```
