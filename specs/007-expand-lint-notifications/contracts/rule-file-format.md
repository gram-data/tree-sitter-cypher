# Contract: `.scm` Rule File Format

**Version**: 2.0 (adds `Code:` header)

## Format

```
;; Rule: <name>
;; Severity: <level>
;; Applies-to: <scope>
;; Message: <text>
[;; Code: <neo4j-notification-code>]   ← optional
<tree-sitter-s-expression-query>
```

## Header Fields

| Header | Required | Values | Description |
|--------|----------|--------|-------------|
| `Rule:` | yes | any string | Unique rule name; used as `diagnostic.rule` |
| `Severity:` | yes | `Error`, `Warning`, `Information`, `Hint` | Diagnostic severity |
| `Applies-to:` | yes | `structural`, `contract`, `cross-reference` | Rule category |
| `Message:` | yes | any string | Human-readable diagnostic message |
| `Code:` | no | e.g. `03N90` | Neo4j GQLSTATUS notification code |

## Behaviour

- Headers are parsed in order; unrecognised `;;` lines are ignored.
- The query begins at the first non-`;;`-prefixed, non-blank line.
- A missing `Code:` header sets `diagnostic.code` to `null` (omitted from JSON output).
- A present `Code:` header sets `diagnostic.code` to the trimmed value string.

## Example

```scheme
;; Rule: CartesianProduct
;; Severity: Warning
;; Applies-to: structural
;; Message: Disconnected MATCH patterns produce a cartesian product.
;; Code: 03N90
(match_clause
  pattern: (pattern
    (path_pattern)
    (path_pattern) @hit))
```

## Compatibility

- Rule files without a `Code:` header remain valid — `code` is optional and backward-compatible.
- The `--rules-dir` external rule mechanism respects the same format; user-supplied rules may include custom codes.
