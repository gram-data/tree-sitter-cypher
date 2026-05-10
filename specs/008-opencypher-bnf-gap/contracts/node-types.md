# Parse-Tree Contract: New Node Types

**Branch**: `008-opencypher-bnf-gap` | **Date**: 2026-05-10

This document defines the public parse-tree contract for every new named node type introduced
by this feature. Consumers (lint rules, syntax highlighters, editor integrations) MUST rely only
on the node names and child fields listed here.

The generated `src/node-types.json` is the authoritative machine-readable version. This document
describes the intended shape before generation.

---

## `legacy_shortest_path_pattern`

Represents a `shortestPath(...)` or `allShortestPaths(...)` expression.

```
(legacy_shortest_path_pattern
  ; anonymous keyword (shortestpath or allshortestpaths) — no field label
  start: (node_pattern)
  (relationship_pattern)
  end: (node_pattern))
```

The leading keyword token is anonymous; distinguish `shortestPath` from `allShortestPaths` by reading the node's text content, not a field.

**Example**:
```cypher
MATCH path = shortestPath((a:Person)-[*]-(b:Person))
```

---

## `map_projection`

Represents a `variable { ... }` map projection expression.

```
(map_projection
  object: (identifier | escaped_identifier)   ; variable reference only, not arbitrary expression
  (map_projection_element)*)
```

**Example**:
```cypher
RETURN n { .name, .age, score: 10, .* }
```

---

## `map_projection_element`

A union of four concrete shapes:

### `field_selector`
```
(field_selector
  property: (identifier | escaped_identifier))
```
Example: `.name`, `.`first-name``

### `all_fields_selector`
```
(all_fields_selector)   ; leaf — matches .*
```
Example: `.*`

### `literal_map_field`
```
(literal_map_field
  key: (identifier)
  value: (expression))
```
Example: `score: 10`

### `variable_selector`
```
(variable_selector
  variable: (identifier))
```
Example: `n` (bare variable in projection)

---

## `path_search_prefix`

Appears as the optional first child of `match_clause` when a GQL path-search mode is specified.
A union of six concrete shapes:

| Node type | Example |
|---|---|
| `all_path_search` | `ALL PATHS` |
| `any_path_search` | `ANY 3 PATHS` |
| `all_shortest_path_search` | `ALL SHORTEST` |
| `any_shortest_path_search` | `ANY SHORTEST` |
| `counted_shortest_path_search` | `SHORTEST 3` |
| `counted_shortest_group_search` | `SHORTEST 3 GROUPS` |

**Example**:
```cypher
MATCH ANY SHORTEST (a:Person)-[*]-(b:Person) RETURN a, b
```

---

## `quantified_path_primary`

Represents a parenthesized sub-path with a repetition quantifier.

```
(quantified_path_primary
  (node_pattern)
  (relationship_pattern (node_pattern))+
  quantifier: (graph_pattern_quantifier))
```

**Example**:
```cypher
MATCH ((a)-[:KNOWS]->(b)){1,3} RETURN a, b
```

---

## `graph_pattern_quantifier`

The quantifier attached to a `quantified_path_primary`. One of:

| Node type | Example |
|---|---|
| `+` token | `+` |
| `*` token | `*` |
| `fixed_quantifier` | `{3}` |
| `general_quantifier` | `{1,3}`, `{2,}`, `{,5}` |

---

## `fixed_quantifier`

```
(fixed_quantifier
  count: (integer_literal))
```

---

## `general_quantifier`

```
(general_quantifier
  lower: (integer_literal)?
  upper: (integer_literal)?)
```

---

## `inf_literal` / `infinity_literal` / `nan_literal`

Case-insensitive keyword leaf nodes. No children.

```
(inf_literal)       ; matches INF, Inf, inf
(infinity_literal)  ; matches INFINITY, infinity, etc.
(nan_literal)       ; matches NAN, nan, NaN
```

---

## Modified existing nodes

These nodes gain a new optional child but retain all previous children:

| Node | New child | Position |
|---|---|---|
| `node_pattern` | `where_clause` (optional) | Last, before `)` |
| `relationship_body` | `where_clause` (optional) | Last, before end of body |
| `yield_clause` | `where_clause` (optional) | After yield-item list |

---

## Stability guarantee

Node names listed in this document are stable once the feature merges. Renaming a node type
is a breaking change requiring a minor version bump of the grammar package.
