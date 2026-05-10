# Data Model: New Grammar Rules

**Branch**: `008-opencypher-bnf-gap` | **Date**: 2026-05-10

This document lists every new named node type added to the parse tree, its BNF anchor, its
children, and the existing parent rules that gain it as a child.

---

## Slice 1 — Shortest Path

### `legacy_shortest_path_pattern`

**BNF**: `<legacy_shortest_path_pattern>`

| Child field | Type | Cardinality |
|---|---|---|
| `function` | anonymous keyword (`shortestpath` or `allshortestpaths`) | 1 |
| — | `node_pattern` | 1 (start node) |
| — | `relationship_pattern` | 1 |
| — | `node_pattern` | 1 (end node) |

**Parent rules updated**: `expression` (new alternative)

---

## Slice 2a — Inline WHERE in Patterns

No new named rules. Existing rules gain an optional child:

| Rule modified | New optional child | Position |
|---|---|---|
| `node_pattern` | `where_clause` | last, before `)` |
| `relationship_body` (all branches) | `where_clause` | last, before end of body |

---

## Slice 2b — Map Projection

### `map_projection`

**BNF**: `<map_projection>`

| Child field | Type | Cardinality |
|---|---|---|
| `object` | `expression` | 1 |
| — | `map_projection_element` (via commaSep) | 0..N |

**Parent rules updated**: `expression` (new alternative), `conflicts` (+ `map_literal`)

---

### `map_projection_element`

**BNF**: `<map_projection_element>`

A choice node — exactly one of the four sub-types below:

| Sub-type rule | Shape | BNF anchor |
|---|---|---|
| `field_selector` | `'.' identifier` | `<field_selector>` |
| `all_fields_selector` | `'.' '*'` | `<all_fields_selector>` |
| `literal_map_field` | `identifier ':' expression` | `<literal_map_field>` |
| `variable_selector` | `identifier` | `<variable_selector>` |

**Parent rules updated**: `map_projection` (child via `commaSep`)

---

### `field_selector`

**BNF**: `<field_selector>`

| Child | Type | Cardinality |
|---|---|---|
| — | `.` token | 1 |
| `property` | `identifier` | 1 |

---

### `all_fields_selector`

**BNF**: `<all_fields_selector>`

Leaf node matching `'.' '*'`. No named children.

---

### `literal_map_field`

**BNF**: `<literal_map_field>`

| Child field | Type | Cardinality |
|---|---|---|
| `key` | `identifier` | 1 |
| `value` | `expression` | 1 |

---

### `variable_selector`

**BNF**: `<variable_selector>`

Wraps a single `identifier`. One named child (`variable`).

---

## Slice 3a — GQL Path-Search Prefixes

### `path_search_prefix`

**BNF**: `<path_search_prefix>`

A choice node — one of the concrete prefix types:

| Sub-type rule | Keyword sequence | BNF anchor |
|---|---|---|
| `all_path_search` | `ALL [PATH\|PATHS]` | `<all_path_search>` |
| `any_path_search` | `ANY [n] [PATH\|PATHS]` | `<any_path_search>` |
| `all_shortest_path_search` | `ALL SHORTEST [PATH\|PATHS]` | `<all_shortest_path_search>` |
| `any_shortest_path_search` | `ANY SHORTEST [PATH\|PATHS]` | `<any_shortest_path_search>` |
| `counted_shortest_path_search` | `SHORTEST n [PATH\|PATHS]` | `<counted_shortest_path_search>` |
| `counted_shortest_group_search` | `SHORTEST [n] [PATH\|PATHS] GROUPS\|GROUP` | `<counted_shortest_group_search>` |

**Parent rules updated**: `match_clause` (optional first child after `MATCH` keyword)

---

### Concrete prefix rules

Each is a simple sequence of keyword tokens and optional `integer_literal`. No nested named children
beyond the count for `any_path_search` / `counted_shortest_path_search`.

---

## Slice 3b — Quantified Path Patterns

### `quantified_path_primary`

**BNF**: `<quantified_path_primary>`

| Child | Type | Cardinality |
|---|---|---|
| — | `node_pattern` | 1 (start) |
| — | `relationship_pattern` + `node_pattern` pairs | 1..N |
| `quantifier` | `graph_pattern_quantifier` | 1 |

**Parent rules updated**: `path_pattern` (new alternative alongside the existing linear path)

---

### `graph_pattern_quantifier`

**BNF**: `<graph_pattern_quantifier>`

A choice of: `+` token, `*` token, `fixed_quantifier`, or `general_quantifier`.

---

### `fixed_quantifier`

**BNF**: `<fixed_quantifier>`

`'{' integer_literal '}'`. One named child (`count`).

---

### `general_quantifier`

**BNF**: `<general_quantifier>`

`'{' optional(integer_literal) ',' optional(integer_literal) '}'`.

| Child field | Type | Cardinality |
|---|---|---|
| `lower` | `integer_literal` | 0..1 |
| `upper` | `integer_literal` | 0..1 |

---

## Slice 3c — YIELD … WHERE

No new named rules. `yield_clause` gains an optional `where_clause` child after the yield-item list.

---

## Slice 4 — Numeric Literal Extensions

### `inf_literal`

**BNF**: `<signed_numeric_literal>` (INF form)

Keyword token matching `/[Ii][Nn][Ff]/`. Leaf node. Listed in `expression` before `identifier`.

---

### `infinity_literal`

**BNF**: `<signed_numeric_literal>` (INFINITY form)

Keyword token matching `/[Ii][Nn][Ff][Ii][Nn][Ii][Tt][Yy]/`. Leaf node.

---

### `nan_literal`

**BNF**: `<signed_numeric_literal>` (NAN form)

Keyword token matching `/[Nn][Aa][Nn]/`. Leaf node.

---

### Modified rules (no new type)

| Rule | Change |
|---|---|
| `float_literal` | Regex extended: optional `[fFdD]` suffix after the number |
| `integer_literal` | Regex extended: `_` allowed between digit groups |
| `float_literal` | Regex extended: `_` allowed between digit groups |
