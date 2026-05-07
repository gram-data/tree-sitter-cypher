# Proposal: Cypherdoc Injection Grammar

**Status**: Draft
**Location**: `tree-sitter-cypherdoc/` (subdirectory of this repo)

## Overview

Cypherdoc is a structured documentation comment format for Cypher statements, modelled on
JSDoc/TSDoc conventions. A cypherdoc comment names a Cypher statement, describes what it does,
declares its parameters, and describes its return shape — enough information to drive agent tool
registration, API generation, or a Cypher linter.

The grammar lives in `tree-sitter-cypherdoc/` as a self-contained Tree-sitter grammar and is
parsed via Tree-sitter's injection mechanism: `tree-sitter-cypher` matches `doc_comment` nodes
and injects the `cypherdoc` language into their text.

---

## Format

A cypherdoc comment is a `/** ... */` block. The leading ` * ` decoration on each line is
treated as whitespace by the grammar.

```
/**
 * <name>
 *
 * <description>
 *
 * @param {<type>} <name> - <description>
 * @param {<type>} [<name>=<default>] - <description> (optional param)
 * @returns {[<col>: <type>, ...]} - <description>    (one row)
 * @returns {[<col>: <type>, ...][]} - <description>  (many rows)
 */
```

**Name** — the first non-empty content line; a snake_case or camelCase identifier that names
the statement as an agent tool.

**Description** — free prose between the name and the first tag. May span multiple paragraphs.

**`@param`** — one entry per Cypher `$parameter`. The type is a scalar cypherdoc type. Square
brackets mark the parameter as optional; a default value is required for all optional params.
This ensures Cypher never receives an undefined binding.

**`@returns`** — a single entry describing the complete row shape as a named tuple type. The
outer `[]` suffix signals that the query returns multiple rows; its absence signals at most one.

### Examples

```cypher
/**
 * find_person_by_name
 *
 * Find a Person node by exact name match.
 *
 * @param {string} name - The full name to search for
 * @returns {[person: node<Person>]} - The matching person, or no rows if not found
 */
MATCH (person:Person {name: $name})
RETURN person
```

```cypher
/**
 * get_colleagues
 *
 * Find people who work at the same company as the given person.
 *
 * @param {string} name - Full name of the person
 * @param {integer} [limit=25] - Maximum number of results to return
 * @returns {[colleague_name: string, company: string][]} - One row per colleague found
 */
MATCH (p:Person {name: $name})-[:WORKS_AT]->(c:Company)<-[:WORKS_AT]-(colleague:Person)
RETURN colleague.name AS colleague_name, c.name AS company
ORDER BY colleague_name
LIMIT $limit
```

---

## Type System

Cypherdoc types map directly to Cypher's value types.

### Scalar types

| Type | Description |
|---|---|
| `string` | UTF-8 string |
| `integer` | 64-bit signed integer |
| `float` | 64-bit IEEE 754 float |
| `boolean` | `true` or `false` |
| `node` | Graph node (any labels) |
| `node<Label>` | Graph node constrained to a label |
| `relationship` | Graph relationship (any type) |
| `relationship<TYPE>` | Graph relationship constrained to a type |
| `path` | Sequence of nodes and relationships |
| `list<type>` | Homogeneous list of a scalar type |
| `map` | Key-value map |
| `any` | Unconstrained / unknown |

### Tuple types (for `@returns` only)

| Syntax | Meaning |
|---|---|
| `[col: type, ...]` | Named tuple — one row |
| `[col: type, ...][]` | Array of named tuples — many rows |

---

## AST

The grammar produces the following node types.

```
doc_comment
  name                      "find_person_by_name"
  description               free prose text
  param_tag                 one per @param
    type_annotation         {string}
      scalar_type           string
    required_param          name
      OR
    optional_param          [name=default]
      param_default         default literal value
    tag_description         "- The full name to search for"
  returns_tag               one @returns
    returns_type_annotation {[person: node<Person>]} or {[...][]}
      tuple_type            [col: type, ...]
        tuple_member        col: type
          identifier        col
          scalar_type       node<Person>
            type_argument   Person
        array_marker        [] (present = many rows, child of tuple_type)
    tag_description         "- The matching person..."
```

### Key design decisions

- `required_param` and `optional_param` are distinct node types so consumers can detect
  optionality without inspecting child tokens.
- `array_marker` is a named child of `tuple_type` (not `type_annotation`). Defining
  `"[]"` as a single atomic two-character token avoids ambiguity between the tuple's
  closing `]` and the array suffix `[]`.
- `tag_description` captures everything after the `-` separator to end-of-line, including
  the dash, so it can be extracted verbatim or stripped.
- `name` and `description` are captured as named nodes (not anonymous tokens) so query
  files can address them directly: `(doc_comment (name) @name)`.

---

## Injection wiring

`tree-sitter-cypher` already contains the injection hook in `queries/injections.scm`:

```scheme
((doc_comment) @injection.content
  (#set! injection.language "cypherdoc"))
```

The cypherdoc grammar receives the full `/** ... */` text of each `doc_comment` node,
including the opening `/**` and closing `*/`. The grammar handles the delimiters and
treats leading ` * ` on each line as whitespace via `extras`.

---

## Directory layout

```
tree-sitter-cypherdoc/
  grammar.js            cypherdoc grammar (source of truth)
  package.json          name: "tree-sitter-cypherdoc"
  tree-sitter.json      grammar metadata
  src/
    grammar.json        generated
    parser.c            generated
    node-types.json     generated
  queries/
    highlights.scm      @name, @param.name, @type, @tag, etc.
    tags.scm            (name) @name for symbol indexing
  test/
    corpus/
      tags.txt          @param and @returns parsing
      types.txt         scalar and tuple type parsing
      names.txt         tool name and description parsing
```

The subdirectory is a fully self-contained Tree-sitter grammar. It can be extracted to its
own repository without modifying anything in `tree-sitter-cypher`.
