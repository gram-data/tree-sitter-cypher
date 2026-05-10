# Data Model: Expand Lint Coverage with Neo4j Notification Codes

**Feature**: 007-expand-lint-notifications
**Date**: 2026-05-10

## Entities

### Rule

A static analysis rule loaded from a `.scm` file and applied to every Cypher parse tree.

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Unique rule identifier (e.g., `"CartesianProduct"`) |
| `severity` | `Severity` | `Error`, `Warning`, `Information`, or `Hint` |
| `applies_to` | `AppliesTo` | `Structural`, `Contract`, or `CrossReference` |
| `message` | `String` | Human-readable description of the issue |
| `code` | `Option<String>` | **NEW** — Neo4j notification code (e.g., `"03N90"`) |
| `query` | `tree_sitter::Query` | Compiled tree-sitter query |

**Change**: Add `code: Option<String>` field. Parsed from new `Code:` header in `.scm` files. Absent header → `None`.

---

### Diagnostic

An output record emitted when a rule fires on a specific source location. Defined in `gram-diagnostics` crate — **no struct changes needed**.

| Field | Type | Notes |
|-------|------|-------|
| `severity` | `Severity` | Copied from `Rule` |
| `rule` | `String` | Copied from `Rule.name` |
| `message` | `String` | Copied from `Rule.message` |
| `range` | `Range` | Source location (line/character, 0-based) |
| `code` | `Option<String>` | **Populated** from `Rule.code` (was always `None` before) |

---

### `.scm` Rule File Format (updated)

```scheme
;; Rule: RuleName
;; Severity: Warning|Error|Information|Hint
;; Applies-to: structural|contract|cross-reference
;; Message: Human-readable message text.
;; Code: NNNXX                   ← NEW optional header
(tree-sitter-query-pattern @capture)
```

The `Code:` header is optional. When present, its value is stored on the `Rule` and copied into every `Diagnostic` emitted by that rule.

---

## New Rule Files

### `rules/structural/cartesian_product.scm`

Detects a `match_clause` pattern with two or more `path_pattern` children (disconnected MATCH patterns).

```
;; Rule: CartesianProduct
;; Severity: Warning
;; Applies-to: structural
;; Message: Disconnected MATCH patterns produce a cartesian product. Connect the patterns with a relationship or split into separate MATCH clauses.
;; Code: 03N90
```

### `rules/structural/deprecated_id_function.scm`

Detects calls to the deprecated `id()` function.

```
;; Rule: DeprecatedFunction
;; Severity: Warning
;; Applies-to: structural
;; Message: id() is deprecated in Neo4j 5. Use elementId() instead.
;; Code: 01N01
```

### `rules/structural/dynamic_property.scm`

Detects dynamic property access `n[$key]` or `n[variable]` that prevents index usage.

```
;; Rule: DynamicProperty
;; Severity: Information
;; Applies-to: structural
;; Message: Dynamic property key prevents index use. Consider using a static property name if the key is known.
;; Code: 03N95
```

---

## Validation Rules

- A `Rule` with a non-`None` `code` MUST have a code matching the pattern `[0-9]{2}[A-Z][0-9]{2}` (Neo4j GQLSTATUS format).
- The `Code:` header value is stored as-is; no validation of the code value is performed at rule load time (avoids coupling to a Neo4j code registry).
