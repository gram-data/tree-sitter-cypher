# Contract: Cypherdoc Query Patterns

The public interface of `tree-sitter-cypherdoc` is its AST. Consumers (editors, linters,
code generators, agent tool registrars) interact with it exclusively through Tree-sitter
query patterns (`.scm` files) or the Tree-sitter node API. This document specifies the
canonical query patterns for each use case.

---

## Extract tool name

```scheme
(document
  (name) @tool.name)
```

**Guarantees**: `@tool.name` is a single token matching `[a-zA-Z_][a-zA-Z0-9_]*`.
Present 0–1 times per document.

---

## Extract description

```scheme
(document
  (description) @tool.description)
```

**Guarantees**: `@tool.description` is a node composed of `description_line` children.
Present 0–1 times per document.

---

## Extract all parameters

```scheme
(param_tag
  type: (type_annotation
    type: (scalar_type
      name: (identifier) @param.type))
  param: (required_param
    name: (identifier) @param.name)
  description: (tag_description) @param.description)
```

```scheme
(param_tag
  type: (type_annotation
    type: (scalar_type
      name: (identifier) @param.type))
  param: (optional_param
    name: (identifier) @param.name
    default: (param_default) @param.default)
  description: (tag_description) @param.description)
```

**Guarantees**: Required and optional params are distinct node types. To match both, use
two separate patterns or `(choice ...)`. `@param.default` is present only for optional params.

---

## Distinguish required vs optional parameters

```scheme
(param_tag
  param: (required_param) @param.required)

(param_tag
  param: (optional_param) @param.optional)
```

---

## Extract type argument (e.g. `node<Person>`)

```scheme
(scalar_type
  name: (identifier) @type.name
  argument: (type_argument
    value: (identifier) @type.argument))
```

**Guarantees**: `@type.argument` is present only when the type has a `<Label>` or `<TYPE>`
qualifier. For unqualified types (e.g., `string`, `path`), only `@type.name` matches.

---

## Extract return tuple shape (one row)

```scheme
(returns_tag
  type: (type_annotation
    type: (tuple_type) @returns.tuple)
  description: (tag_description) @returns.description)

(tuple_type
  (tuple_member
    column: (identifier) @column.name
    type: (scalar_type
      name: (identifier) @column.type)))
```

**To check cardinality**: The presence of `(array_marker)` as a child of `tuple_type`
signals many rows:

```scheme
(tuple_type
  (array_marker) @returns.many)
```

If this pattern matches, the result is a list. If it does not match, the result is a single
row (or no rows).

---

## Full agent tool extraction (composed pattern)

To extract everything needed for agent tool registration in one pass:

```scheme
; Tool name
(document (name) @tool.name)

; Description
(document (description) @tool.description)

; Required params
(param_tag
  type: (type_annotation type: (scalar_type name: (identifier) @param.type))
  param: (required_param name: (identifier) @param.name)
  description: (tag_description) @param.description)

; Optional params
(param_tag
  type: (type_annotation type: (scalar_type name: (identifier) @param.type))
  param: (optional_param
    name: (identifier) @param.name
    default: (param_default) @param.default)
  description: (tag_description) @param.description)

; Return columns
(tuple_member
  column: (identifier) @column.name
  type: (scalar_type name: (identifier) @column.type))

; Return cardinality
(tuple_type (array_marker) @returns.many)
```

---

## Injection hook (in `tree-sitter-cypher`)

```scheme
; queries/injections.scm — already present in tree-sitter-cypher
((doc_comment) @injection.content
  (#set! injection.language "cypherdoc"))
```

**Guarantees**: Every `doc_comment` node in a parsed Cypher file triggers cypherdoc parsing.
The full `/** ... */` text (including delimiters) is passed to the cypherdoc grammar.

---

## Invariants

1. A `document` with no `name` node is valid (comment-only block).
2. A `document` with no `returns_tag` is valid (e.g., a named description-only entry).
3. `optional_param` always has exactly one `param_default` child — bare `[name]` is a
   parse error.
4. `returns_tag` always uses a `tuple_type`, never a bare `scalar_type`.
5. `param_tag` always uses a `scalar_type`, never a `tuple_type`.
6. `array_marker` appears only as a child of `tuple_type`, never standalone.
