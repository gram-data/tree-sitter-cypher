# Research: Expand Lint Coverage with Neo4j Notification Codes

**Date**: 2026-05-10
**Feature**: 007-expand-lint-notifications

## AST Node Shapes (Verified via `tree-sitter parse`)

### CartesianProduct (03N90)

A `MATCH` with comma-separated disconnected patterns produces a `pattern` node containing multiple `path_pattern` children:

```
(match_clause
  pattern: (pattern
    (path_pattern ...)        ← first connected pattern
    (path_pattern ...) @hit   ← second = cartesian product
```

**Decision**: Match any `match_clause` whose `pattern` has at least two `path_pattern` children. Flag the second (and later) disconnected patterns.

**Rationale**: This matches what Neo4j's own planner flags — any comma-separated MATCH is a candidate cartesian product. Static analysis cannot determine whether variables overlap; flagging conservatively aligns with Neo4j's notification behaviour.

**Alternatives considered**: Attempting to detect shared variable names across path patterns. Rejected — tree-sitter queries cannot resolve variable binding; this would require a multi-pass analyser.

---

### DeprecatedFunction: `id()` → `elementId()` (01N01)

A bare `id(n)` call produces:
```
(function_call
  name: (function_name
    (identifier) "id")   ← single identifier, no dots
  (expression ...))
```

A qualified name like `apoc.id()` produces `(function_name (identifier "apoc") (identifier "id"))`.

**Decision**: Match `function_call` where `function_name` has exactly one `identifier` child equal to `"id"` (case-insensitive matching to follow).

**Rationale**: The `id()` deprecation is in Neo4j 5 and is the most impactful deprecated-function change. Starting with `id()` covers the primary case; other deprecated functions can be added in later rules.

**Alternatives considered**: Matching all deprecated functions in one rule. Rejected — each function has its own code and message; separate rules are cleaner and independently suppressible.

---

### DynamicProperty (03N95)

A `subscript_expression` can represent either a list index or a dynamic property access. The inner expression distinguishes them:

| Syntax       | Inner expression node  | Flag? |
|--------------|------------------------|-------|
| `n[$param]`  | `parameter`            | Yes   |
| `n[variable]`| `identifier`           | Yes   |
| `n[fn()]`    | `function_call`        | Yes   |
| `n[0]`       | `integer_literal`      | No    |
| `n["name"]`  | `string_literal`       | No    |
| `list[1..3]` | slice syntax (two `..`)| No    |

**Decision**: Implement as two rules (or one rule with two `#match?` branches):
1. Subscript expression where key is `parameter`
2. Subscript expression where key is a bare `identifier`

Function-call keys (e.g., `n[toLower("X")]`) can be added in a follow-up.

**Rationale**: Parameters and bare identifiers are the most common dynamic-key patterns. Static string/integer keys are provably not dynamic.

**Alternatives considered**: Matching all non-literal subscripts with `#not-match?`. Harder to express cleanly; the positive match approach is readable and correct.

---

### DeprecatedRelationshipTypeList `[:A|:B]` (01N01) — FINDING: Already a ParseError

**Finding**: `[:FOO|:BAR]` produces a **MISSING node** in the current grammar, which the existing `collect_error_nodes` function already flags as a `ParseError`. The grammar parses `[:FOO|:BAR]` by:
1. Consuming the leading `:` as `label_expression` start
2. Parsing `FOO` as the first label
3. Seeing `|` and expecting an identifier for the right-hand side of `|`
4. `:` is unexpected — tree-sitter inserts a MISSING identifier and continues
5. The trailing `:BAR` is consumed by `label_expression`'s `repeat(: inner)` handler

**Decision**: Remove `DeprecatedRelationshipTypeList` from this feature's scope. It is already caught as a `ParseError` with "Syntax error in Cypher query." A separate future feature could improve ParseError messages to be contextually specific.

**Rationale**: Adding a dedicated lint rule that fires on MISSING nodes in this specific context is fragile and overlaps with the ParseError path. The user already gets an error; the only improvement would be a better message, which requires a different mechanism.

---

## `Rule` Struct — `code` Field Support

The `Diagnostic` struct (in `gram-diagnostics`) already has `code: Option<String>` with `skip_serializing_if`. The `Rule` struct in `tools/cypher/src/rules.rs` does **not** have a `code` field, and `make_diagnostic` always sets `code: None`.

**Decision**: Add an optional `Code:` header to `.scm` rule files and a `code: Option<String>` field to `Rule`. Update `make_diagnostic` to copy the rule's code into the diagnostic.

**Implementation path**:
1. Add `pub code: Option<String>` to `Rule` struct
2. Parse `Code: NNXXX` from `.scm` header in `parse_rule_file`
3. Pass `code: rule.code.clone()` in `make_diagnostic`
4. Add `Code:` headers to all new rules; existing rules remain `code: None`

---

## Scope: Three New Rules

Based on research findings, this feature delivers **three** new rules (not four as originally scoped):

| Rule | Code | Severity | BNF anchor |
|------|------|----------|------------|
| `CartesianProduct` | `03N90` | Warning | `<pattern>` production |
| `DeprecatedFunction` | `01N01` | Warning | `<function invocation>` |
| `DynamicProperty` | `03N95` | Information | `<subscript operator>` |

`DeprecatedRelationshipTypeList` is dropped — already covered by `ParseError`.
