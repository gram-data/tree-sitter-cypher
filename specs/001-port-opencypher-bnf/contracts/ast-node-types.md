# Contract: tree-sitter-cypher AST Node Types

**Version**: 0.1.0 (in development)
**Generated from**: `src/node-types.json` after `tree-sitter generate`

This document defines the public contract between the `tree-sitter-cypher` grammar and its consumers (editors, linters, formatters, analyzers using Node, Rust, Python, Go, Java, Swift, or Zig bindings).

---

## Contract Rules

1. **Named nodes are the public API.** Every node type listed in this document corresponds to a rule in `grammar.js` that will appear by name in the parse tree. Consumers may reliably query for these node types.

2. **Anonymous nodes (keywords, punctuation) are structural only.** Keywords like `MATCH`, `WHERE`, `->` and punctuation like `(`, `)`, `,` are anonymous tokens in the tree. Consumers should not depend on their position by index; use named children instead.

3. **The root node is always `source_file`.** A valid parse always produces a `source_file` node containing one or more `statement` children.

4. **The presence of an `ERROR` node indicates invalid input.** Consumers may check for `node.hasError()` to detect parse failures. Tree-sitter performs error recovery, so a partial tree is always produced.

5. **`statement` children are either `match_clause` sequences or `union_statement`.** A linear statement is a sequence of clauses ending with a `return_clause` or a mutation clause. A composite statement is a `union_statement`.

---

## Top-level Shape

```
source_file
  statement+
    match_clause?
    optional_match_clause?
    with_clause?
    unwind_clause?
    create_clause?
    merge_clause?
    set_clause?
    remove_clause?
    delete_clause?
    call_clause?
    return_clause?
  | union_statement
      statement
      statement
```

---

## Named Child Fields

Key nodes expose named fields (accessible as `node.childForFieldName('field')`) to make navigation unambiguous:

| Node | Field | Child type |
|---|---|---|
| `match_clause` | `pattern` | `pattern` |
| `match_clause` | `where` | `where_clause` (optional) |
| `node_pattern` | `variable` | `identifier` (optional) |
| `node_pattern` | `label` | `label_expression` (optional) |
| `node_pattern` | `properties` | `property_map` (optional) |
| `relationship_pattern` | `variable` | `identifier` (optional) |
| `relationship_pattern` | `label` | `label_expression` (optional) |
| `relationship_pattern` | `length` | `path_length` (optional) |
| `relationship_pattern` | `properties` | `property_map` (optional) |
| `return_item` | `expression` | any expression node |
| `return_item` | `alias` | `identifier` (optional, the AS name) |
| `property_access` | `object` | expression node |
| `property_access` | `property` | `identifier` |
| `function_call` | `name` | `function_name` |
| `function_call` | `arguments` | expression nodes |
| `binary_expression` | `left` | expression node |
| `binary_expression` | `operator` | anonymous token |
| `binary_expression` | `right` | expression node |
| `case_expression` | `operand` | expression node (optional — simple CASE only) |
| `case_expression` | `when_clauses` | `case_when_clause`+ |
| `case_expression` | `else_clause` | `case_else_clause` (optional) |
| `merge_action` | `trigger` | `MATCH` or `CREATE` (anonymous) |
| `merge_action` | `set_clause` | `set_clause` |
| `yield_item` | `name` | `identifier` |
| `yield_item` | `alias` | `identifier` (optional) |

---

## Stability Guarantees

- Node type names listed here are **stable** once a slice is shipped (no renames without a major version bump).
- Field names are **stable** once defined.
- The set of named node types may **grow** as later slices (P5–P8) are implemented — additive changes are non-breaking.
- The set of named node types will **not shrink** within a minor version.
