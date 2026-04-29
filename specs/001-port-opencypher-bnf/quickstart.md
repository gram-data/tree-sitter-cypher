# Quickstart: Authoring the tree-sitter-cypher Grammar

## The development loop

```sh
# After every change to grammar.js:
tree-sitter generate

# Run all corpus tests:
tree-sitter test

# Run one test file by name filter:
tree-sitter test -f "literals"

# Inspect the parse tree for a file:
tree-sitter parse example-file

# Interactive playground (wasm build required):
npm start
```

## grammar.js skeleton

```js
/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

// Helper: case-insensitive keyword terminal
const kw = str =>
  token(new RegExp(
    str.split('').map(c =>
      /[a-zA-Z]/.test(c) ? `[${c.toUpperCase()}${c.toLowerCase()}]` : c
    ).join('')
  ));

// Helper: comma-separated lists
const commaSep1 = rule => seq(rule, repeat(seq(',', rule)));
const commaSep  = rule => optional(commaSep1(rule));

export default grammar({
  name: 'cypher',

  extras: $ => [
    /\s+/,
    /\/\/.*/,
    /\/\*[^*]*\*+([^/*][^*]*\*+)*\//,
  ],

  rules: {
    source_file: $ => repeat1($.statement),
    // ... add rules here
  },
});
```

## Adding a corpus test

Create or extend a file in `test/corpus/<slice>.txt`:

```text
======================
Simple MATCH RETURN
======================

MATCH (n) RETURN n

---

(source_file
  (statement
    (match_clause
      (pattern
        (path_pattern
          (node_pattern
            (identifier)))))
    (return_clause
      (return_body
        (return_item
          (identifier))))))
```

Each test block has:
1. A line of `=` characters with a title
2. The Cypher input
3. A `---` separator
4. The expected s-expression tree

## BNF → grammar.js translation reference

| BNF | grammar.js |
|---|---|
| `A B` | `seq(A, B)` |
| `A \| B` | `choice(A, B)` |
| `[ A ]` | `optional(A)` |
| `{ A }...` | `repeat1(A)` |
| `[ { A }... ]` | `repeat(A)` |
| `A [ { , A }... ]` | `commaSep1(A)` |
| named rule `<foo>` | `$.foo` (rule reference) |
| terminal string `MATCH` | `kw('MATCH')` |
| punctuation `(` | `'('` (string literal) |

## Operator precedence

Use `prec.left(level, ...)` for left-associative operators. Higher numbers bind tighter:

```js
// OR binds loosest
or_expression: $ => prec.left(1, seq($.expression, kw('OR'), $.expression)),
// AND binds tighter than OR
and_expression: $ => prec.left(3, seq($.expression, kw('AND'), $.expression)),
// Postfix binds tightest
property_access: $ => prec.left(10, seq($.expression, '.', $.identifier)),
```

## BNF reference

The authoritative spec is `references/openCypher/grammar/openCypher.bnf` (1,533 lines).
Top-level sections and their grammar.js rule targets:

| BNF section | Key rules |
|---|---|
| Program / Statement | `source_file`, `statement` |
| Query statements | `match_clause`, `with_clause`, `unwind_clause` |
| Data update statements | `create_clause`, `set_clause`, `remove_clause`, `delete_clause` |
| Call procedure | `call_clause`, `yield_clause` |
| Result statements | `return_clause` |
| Common elements | `where_clause`, `order_by_clause`, `skip_clause`, `limit_clause` |
| Patterns | `pattern`, `path_pattern`, `node_pattern`, `relationship_pattern` |
| Label expressions | `label_expression` |
| Catalog references | `procedure_name`, `function_name` |
| Expressions | `binary_expression`, `unary_expression`, `function_call`, etc. |
| Value specifications | `integer_literal`, `float_literal`, `string_literal`, etc. |
| Basics | `identifier`, `escaped_identifier`, `parameter` |
