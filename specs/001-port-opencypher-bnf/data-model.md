# Data Model: AST Node Types for tree-sitter-cypher

This file catalogues the named AST node types that `grammar.js` will produce, organized by the 8 implementation slices. These become the public API surfaced in `src/node-types.json` and consumed by editors, linters, and analysis tools.

---

## P1 — Literals, Identifiers, Comments

| Node type | Description | Example |
|---|---|---|
| `integer_literal` | Decimal integer | `42` |
| `hex_literal` | Hexadecimal integer | `0x2A` |
| `octal_literal` | Octal integer | `0o52` |
| `float_literal` | Floating-point number | `3.14`, `1.5e10` |
| `string_literal` | Single- or double-quoted string | `'hello'`, `"world"` |
| `boolean_literal` | `true` or `false` (case-insensitive) | `TRUE`, `false` |
| `null_literal` | `null` (case-insensitive) | `NULL` |
| `identifier` | Unquoted name | `n`, `Person`, `myVar` |
| `escaped_identifier` | Backtick-quoted name | `` `my var` `` |
| `parameter` | Named or positional parameter | `$name`, `$0` |

---

## P2 — Minimal MATCH/RETURN

| Node type | Description | Example |
|---|---|---|
| `source_file` | Root node; contains one or more statements | *(whole file)* |
| `statement` | A single Cypher statement (may end with `;`) | `MATCH … RETURN …` |
| `match_clause` | A MATCH or OPTIONAL MATCH clause | `MATCH (n)` |
| `return_clause` | A RETURN clause | `RETURN n` |
| `return_item` | A single projected expression, optionally aliased | `n.name AS name` |
| `return_body` | The full projection list (DISTINCT + items + ordering) | `DISTINCT n, m` |
| `node_pattern` | A parenthesized node in a graph pattern | `(n:Person)` |
| `where_clause` | A WHERE filter attached to MATCH or WITH | `WHERE n.active` |

---

## P3 — Graph Patterns

| Node type | Description | Example |
|---|---|---|
| `pattern` | A comma-separated list of path patterns | `(a)-->(b), (c)` |
| `path_pattern` | An optional path variable and a path expression | `p = (a)-->(b)` |
| `node_pattern` | *(extended from P2)* includes labels and properties | `(n:Person {age: 30})` |
| `relationship_pattern` | A directed or undirected relationship | `-[r:KNOWS]->` |
| `relationship_body` | The content inside `[...]` | `r:KNOWS {since: 2020}` |
| `path_length` | A variable-length quantifier | `*`, `*1..3`, `*2` |
| `label_expression` | A boolean combination of labels | `:Person&Employee` |
| `label_name` | A single label or type identifier | `Person` |
| `property_map` | A `{ key: value }` inline property specification | `{name: 'Alice'}` |
| `property_key_value` | A single `key: value` pair in a property map | `name: 'Alice'` |

---

## P4 — Expressions and WHERE

| Node type | Description | Example |
|---|---|---|
| `binary_expression` | An infix operator expression | `a + b`, `x AND y` |
| `unary_expression` | A prefix operator expression | `-n`, `NOT x` |
| `comparison_expression` | A comparison chain | `a > 0 AND a < 10` |
| `property_access` | A dot-notation property lookup | `n.name` |
| `subscript_expression` | A bracket index access | `list[0]` |
| `function_call` | A function invocation | `toUpper(n.name)` |
| `function_name` | The name of a function (may be qualified) | `db.labels` |
| `list_literal` | A `[...]` list value | `[1, 2, 3]` |
| `map_literal` | A `{key: val}` map value | `{x: 1, y: 2}` |
| `is_null_expression` | An `IS NULL` or `IS NOT NULL` test | `n.name IS NULL` |
| `in_expression` | An `IN` list membership test | `x IN [1,2,3]` |
| `starts_with_expression` | `STARTS WITH` string predicate | `n.name STARTS WITH 'A'` |
| `ends_with_expression` | `ENDS WITH` string predicate | `n.name ENDS WITH 'e'` |
| `contains_expression` | `CONTAINS` string predicate | `n.name CONTAINS 'li'` |

---

## P5 — Data Mutation Clauses

| Node type | Description | Example |
|---|---|---|
| `create_clause` | A CREATE clause | `CREATE (n:Person)` |
| `set_clause` | A SET clause | `SET n.name = 'Alice'` |
| `set_item` | A single assignment inside SET | `n.age = 30` |
| `remove_clause` | A REMOVE clause | `REMOVE n:Temp` |
| `remove_item` | A single label or property removal | `n:Label` |
| `delete_clause` | A DELETE or DETACH DELETE clause | `DETACH DELETE n` |

---

## P6 — Pipeline Clauses

| Node type | Description | Example |
|---|---|---|
| `with_clause` | A WITH clause | `WITH n ORDER BY n.name` |
| `unwind_clause` | An UNWIND clause | `UNWIND list AS x` |
| `order_by_clause` | An ORDER BY specification | `ORDER BY n.name DESC` |
| `sort_item` | A single sort expression with optional direction | `n.age ASC` |
| `skip_clause` | A SKIP clause | `SKIP 10` |
| `limit_clause` | A LIMIT clause | `LIMIT 5` |

---

## P7 — MERGE and CALL

| Node type | Description | Example |
|---|---|---|
| `merge_clause` | A MERGE clause | `MERGE (n:Person {id: 1})` |
| `merge_action` | An ON MATCH or ON CREATE action | `ON CREATE SET n.ts = 0` |
| `call_clause` | A CALL procedure invocation | `CALL db.labels()` |
| `procedure_name` | A qualified procedure name | `db.labels` |
| `procedure_argument_list` | The argument list in `(...)` | `(n, 'param')` |
| `yield_clause` | A YIELD projection list | `YIELD label` |
| `yield_item` | A single yielded name with optional alias | `label AS l` |

---

## P8 — UNION and Advanced Expressions

| Node type | Description | Example |
|---|---|---|
| `union_statement` | Two statements joined by UNION [ALL] | `… UNION …` |
| `case_expression` | A CASE expression (simple or searched) | `CASE n.x WHEN 1 THEN 'a' END` |
| `case_when_clause` | A WHEN…THEN pair | `WHEN 1 THEN 'a'` |
| `case_else_clause` | The ELSE branch | `ELSE 'other'` |
| `list_comprehension` | A `[x IN list \| expr]` comprehension | `[x IN xs WHERE x > 0 \| x*2]` |
| `pattern_comprehension` | A `[(pattern) \| expr]` path value | `[(n)-->(m) \| m.name]` |
| `reduce_expression` | A REDUCE accumulator expression | `reduce(s=0, x IN xs \| s + x)` |
| `all_expression` | An `ALL(x IN list WHERE ...)` quantifier | `ALL(x IN xs WHERE x > 0)` |
| `any_expression` | An `ANY(...)` quantifier | `ANY(x IN xs WHERE x > 0)` |
| `none_expression` | A `NONE(...)` quantifier | `NONE(x IN xs WHERE x < 0)` |
| `single_expression` | A `SINGLE(...)` quantifier | `SINGLE(x IN xs WHERE x = 1)` |
| `count_star` | The `count(*)` aggregate | `count(*)` |
