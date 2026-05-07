# Data Model: Cypherdoc AST

All nodes are produced by `tree-sitter-cypherdoc`. Named fields are accessible via
`node.childForFieldName("field")` in any Tree-sitter binding.

---

## Root

### `document`

The root node of every parsed cypherdoc block. Always spans the full `/** ... */` text.

| Child | Field | Cardinality | Notes |
|---|---|---|---|
| `name` | — | 0–1 | First non-empty, non-tag line |
| `description` | — | 0–1 | Prose before first `@` tag |
| `param_tag` | — | 0–N | One per `@param` |
| `returns_tag` | — | 0–1 | The `@returns` entry |

---

## Tag nodes

### `param_tag`

Represents one `@param` entry.

| Child | Field | Cardinality | Notes |
|---|---|---|---|
| `type_annotation` | `type` | 1 | `{scalar_type}` only (not tuple) |
| `required_param` | `param` | 1 (xor) | Plain identifier |
| `optional_param` | `param` | 1 (xor) | Bracketed identifier with default |
| `tag_description` | `description` | 0–1 | `- text to end of line` |

### `returns_tag`

Represents the `@returns` entry.

| Child | Field | Cardinality | Notes |
|---|---|---|---|
| `type_annotation` | `type` | 1 | `{tuple_type}` only (not scalar) |
| `tag_description` | `description` | 0–1 | `- text to end of line` |

---

## Param nodes

### `required_param`

A mandatory parameter. No brackets.

| Child | Field | Cardinality | Notes |
|---|---|---|---|
| `identifier` | `name` | 1 | `[a-zA-Z_][a-zA-Z0-9_]*` |

### `optional_param`

An optional parameter. Always has a default value; bare `[name]` is not valid.

| Child | Field | Cardinality | Notes |
|---|---|---|---|
| `identifier` | `name` | 1 | The parameter name |
| `param_default` | `default` | 1 | The default value literal |

### `param_default`

The literal default value of an optional parameter. One of:

- `string_default` — `"..."` or `'...'`
- `number_default` — integer or decimal, optionally negative
- `boolean_default` — `true` or `false`

---

## Type nodes

### `type_annotation`

A `{ ... }` wrapper around a type expression. Used in both `param_tag` and `returns_tag`.

| Child | Field | Cardinality | Notes |
|---|---|---|---|
| `scalar_type` | `type` | 1 (xor) | In `param_tag` contexts |
| `tuple_type` | `type` | 1 (xor) | In `returns_tag` contexts |

### `scalar_type`

A named Cypher value type, optionally parameterised.

| Child | Field | Cardinality | Notes |
|---|---|---|---|
| `identifier` | `name` | 1 | `string`, `integer`, `node`, etc. |
| `type_argument` | `argument` | 0–1 | `<Label>` or `<TYPE>` |

Valid base names: `string`, `integer`, `float`, `boolean`, `node`, `relationship`,
`path`, `list`, `map`, `any`.

### `type_argument`

The label or type name inside `< >`.

| Child | Field | Cardinality | Notes |
|---|---|---|---|
| `identifier` | `value` | 1 | e.g. `Person`, `KNOWS` |

### `tuple_type`

A named-tuple return shape: `[col: type, ...]`.

| Child | Field | Cardinality | Notes |
|---|---|---|---|
| `tuple_member` | — | 1–N | One per return column |
| `array_marker` | — | 0–1 | Present = many rows; absent = one row |

### `tuple_member`

One column in the return tuple.

| Child | Field | Cardinality | Notes |
|---|---|---|---|
| `identifier` | `column` | 1 | Column alias matching the Cypher RETURN alias |
| `scalar_type` | `type` | 1 | The column's value type |

### `array_marker`

The literal token `[]` appended to a `tuple_type`. Presence signals that the query returns
multiple rows. No fields.

---

## Text nodes

### `name`

The tool identifier — the first non-empty, non-tag content line of the comment. Should be
`snake_case` or `camelCase` to match agent tool naming conventions. No internal structure;
captured as a single token matching `[a-zA-Z_][a-zA-Z0-9_]*`.

### `description`

Free prose text between the `name` and the first `@` tag. Composed of one or more
`description_line` children, each carrying one line of text content after the decorative
` * ` prefix is stripped.

### `tag_description`

The description text following the `-` separator on a `@param` or `@returns` line. Captured
verbatim including the leading `-`, from the dash to the end of the line.

---

## State transitions

`param_tag` nodes are ordered by their appearance in the source. `returns_tag` must appear
after all `param_tag` nodes (enforced by the grammar's rule ordering). There is exactly one
`returns_tag` per `document`; a comment with no `@returns` is valid (e.g., a named
description-only comment).
