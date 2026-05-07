# Data Model: TCK Conformance — New AST Node Types

**Feature**: `004-tck-conformance` | **Date**: 2026-05-07

This document defines the new named node types added to `src/node-types.json` by this feature.
All types are Tree-sitter grammar rule names (snake_case). Each maps directly to a BNF production.

---

## New Named Node Types

### `is_labeled_expression`

Represents a label predicate used as a boolean expression. Matches `n:Person`, `n IS Person`, `n:A&B`.

| Field | Cardinality | Type | BNF anchor |
|-------|-------------|------|-----------|
| `expression` (unnamed child) | 1 | `expression` | `<advanced comparison predicand>` |
| `label` (unnamed child) | 1 | `label_expression` | `<is label expression>` |

BNF: `<is labeled predicate part 2> ::= <is label expression>`

Example tree:
```
(is_labeled_expression
  (identifier "n")
  (label_expression
    label_name: (identifier "Person")))
```

---

### `pattern_comprehension`

Represents a path-pattern comprehension: `[(n)-->() | expr]` or `[p = (n)-->() | expr]`.

| Field | Cardinality | Type | BNF anchor |
|-------|-------------|------|-----------|
| `variable` | 0..1 | `identifier` | `<binding variable>` |
| `pattern` | 1 | `path_pattern` | `<simple path pattern>` |
| `where` | 0..1 | `where_clause` | `<pattern filter>` |
| `projection` | 1 | `expression` | `<pattern projection>` |

BNF: `<pattern comprehension> ::= '[' <pattern source> <pattern filter and projection> ']'`

Example tree:
```
(pattern_comprehension
  variable: (identifier "p")
  pattern: (path_pattern
    (node_pattern variable: (identifier "n"))
    (relationship_pattern)
    (node_pattern))
  projection: (expression (identifier "p")))
```

---

### `exists_expression`

Represents an EXISTS subquery predicate: `EXISTS { ... }`.

| Field | Cardinality | Type | BNF anchor |
|-------|-------------|------|-----------|
| unnamed child | 1 | `pattern` OR `exists_subquery` | `<subquery expression argument>` |

BNF: `<exists expression> ::= EXISTS <left brace> <subquery expression argument> <right brace>`

Example trees:
```
(exists_expression
  (pattern ...))               ; graph pattern form

(exists_expression
  (exists_subquery
    (statement ...)))          ; multi-clause form
```

---

### `exists_subquery`

The body of a multi-clause `exists { MATCH ... RETURN ... }` subquery.

| Field | Cardinality | Type | BNF anchor |
|-------|-------------|------|-----------|
| unnamed children | 1..N | `statement` | `<procedure specification>` |

BNF: `<procedure specification> ::= <statement block> ::= <statement>`

---

### `pattern_predicate`

A path pattern used as a boolean expression: `(n)-->(m)`, `(n)-[:T]-()`.

| Field | Cardinality | Type | BNF anchor |
|-------|-------------|------|-----------|
| unnamed children | 2..N | `node_pattern`, `relationship_pattern` | `<simple path pattern>` |

BNF: `<pattern expression> ::= <simple path pattern>` (as `<boolean primary>`)

Example tree:
```
(pattern_predicate
  (node_pattern variable: (identifier "n"))
  (relationship_pattern)
  (node_pattern))
```

---

## Modified Node Types

### `path_length` (token — extended)

Extended from `* | *N | *N..M | *N..` to also support `*..M` and `*..` (no lower bound).

BNF: `<path length> ::= <asterisk> [ <unsigned decimal integer> | <unsigned decimal integer> <range operator> [ <unsigned decimal integer> ] ]`

All valid forms after this change:
- `*` — any length
- `*N` — exact N hops
- `*N..M` — N to M hops
- `*N..` — at least N hops  
- `*..M` — at most M hops (new)
- `*..` — explicit unbounded (new)

### `relationship_pattern` (extended)

Add `<-->` as a valid direction form alongside `-[]-`, `-->`, `<--`.

---

## Node Types NOT Changed

- `expression` — extended by adding `is_labeled_expression`, `pattern_comprehension`, `exists_expression`, `pattern_predicate` to its `choice(...)` list, but the rule structure is unchanged
- `node_pattern` — parameter-in-pattern fix is a parser behavior fix, not a structural change
- All existing node types (102 rules) — must produce identical trees for all existing corpus test inputs
