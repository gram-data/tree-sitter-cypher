# Data Model: Cypher Syntax Highlighting Capture Taxonomy

**Feature**: 002-syntax-highlighting | **Date**: 2026-04-29

This document maps every Cypher AST node type and token to the tree-sitter capture group it should receive. It is the authoritative reference for implementing `highlights.scm`, `locals.scm`, and `tags.scm`.

---

## Grammar Change: Keyword Alias Map

After applying the `alias()` keyword change in `grammar.js`, each keyword occurrence in a rule produces an anonymous node with the type listed below. The alias is the lowercase canonical form.

| Keyword (user input) | Alias node type | Capture group | Category |
|---------------------|-----------------|---------------|----------|
| MATCH / match / Match | `"match"` | `@keyword` | Clause |
| OPTIONAL MATCH | `"optional"` + `"match"` | `@keyword` | Clause modifier |
| RETURN | `"return"` | `@keyword` | Clause |
| WITH | `"with"` | `@keyword` | Clause |
| WHERE | `"where"` | `@keyword` | Clause |
| CREATE | `"create"` | `@keyword` | Clause |
| MERGE | `"merge"` | `@keyword` | Clause |
| DELETE | `"delete"` | `@keyword` | Clause |
| DETACH | `"detach"` | `@keyword.control` | Modifier |
| SET | `"set"` | `@keyword` | Clause |
| REMOVE | `"remove"` | `@keyword` | Clause |
| UNWIND | `"unwind"` | `@keyword` | Clause |
| CALL | `"call"` | `@keyword` | Clause |
| YIELD | `"yield"` | `@keyword` | Clause |
| UNION | `"union"` | `@keyword` | Clause |
| ALL (union) | `"all"` | `@keyword.control` | Modifier |
| ORDER BY | `"order"` + `"by"` | `@keyword` | Clause modifier |
| SKIP / OFFSET | `"skip"` / `"offset"` | `@keyword` | Clause modifier |
| LIMIT | `"limit"` | `@keyword` | Clause modifier |
| DISTINCT | `"distinct"` | `@keyword.control` | Modifier |
| AS | `"as"` | `@keyword.operator` | Binding |
| ON | `"on"` | `@keyword` | Merge modifier |
| CASE | `"case"` | `@keyword` | Expression |
| WHEN | `"when"` | `@keyword` | Expression |
| THEN | `"then"` | `@keyword` | Expression |
| ELSE | `"else"` | `@keyword` | Expression |
| END | `"end"` | `@keyword` | Expression |
| AND | `"and"` | `@keyword.operator` | Logical op |
| OR | `"or"` | `@keyword.operator` | Logical op |
| XOR | `"xor"` | `@keyword.operator` | Logical op |
| NOT | `"not"` | `@keyword.operator` | Logical op |
| IN | `"in"` | `@keyword.operator` | Predicate |
| IS | `"is"` | `@keyword.operator` | Predicate |
| CONTAINS | `"contains"` | `@keyword.operator` | String predicate |
| STARTS | `"starts"` | `@keyword.operator` | String predicate |
| WITH (STARTS WITH) | `"with"` | `@keyword.operator` | String predicate |
| ENDS | `"ends"` | `@keyword.operator` | String predicate |
| ALL (quantifier) | `"all"` | `@keyword.operator` | Quantifier |
| ANY (quantifier) | `"any"` | `@keyword.operator` | Quantifier |
| NONE | `"none"` | `@keyword.operator` | Quantifier |
| SINGLE | `"single"` | `@keyword.operator` | Quantifier |
| REDUCE | `"reduce"` | `@keyword.operator` | Quantifier |
| ASC / ASCENDING | `"asc"` / `"ascending"` | `@keyword.modifier` | Sort |
| DESC / DESCENDING | `"desc"` / `"descending"` | `@keyword.modifier` | Sort |

---

## Named Node Type → Capture Map (highlights.scm)

| AST Node Type | Field / Context | Capture Group | Notes |
|---------------|-----------------|---------------|-------|
| `string_literal` | — | `@string` | |
| `integer_literal` | — | `@number` | |
| `float_literal` | — | `@number` | |
| `boolean_literal` | — | `@boolean` | |
| `null_literal` | — | `@constant.builtin` | |
| `parameter` | — | `@variable.parameter` | $param or $0 |
| `count_star` | — | `@function` | COUNT(*) as a single token |
| `identifier` | inside `label_expression` | `@type` | label and rel-type names |
| `identifier` | `function_name` → any position | `@function` | |
| `procedure_name` → `identifier` | any position | `@function` | |
| `identifier` | `property_key_value` key position | `@property` | map literal and SET keys |
| `property_access` → `.property` field | — | `@property` | n.name property read |
| `escaped_identifier` | inside `label_expression` | `@type` | backtick-quoted label |
| `identifier` | fallback | `@variable` | graph pattern variable |
| `escaped_identifier` | fallback | `@variable` | backtick-quoted variable |
| `path_length` | — | `@number` | *1..5 range token |
| `binary_expression`.`operator` | — | `@operator` | arithmetic/comparison ops |

---

## Anonymous Token → Capture Map (highlights.scm)

| Token | Capture Group |
|-------|---------------|
| `(`, `)` | `@punctuation.bracket` |
| `[`, `]` | `@punctuation.bracket` |
| `{`, `}` | `@punctuation.bracket` |
| `,` | `@punctuation.delimiter` |
| `;` | `@punctuation.delimiter` |
| `.` | `@punctuation.delimiter` |
| `->`, `<-` | `@operator` |
| `-` (relationship dash) | `@operator` |
| `=`, `<>`, `<`, `>`, `<=`, `>=`, `=~` | `@operator` |
| `+`, `-`, `*`, `/`, `%`, `^` | `@operator` |
| `+=` | `@operator` |
| `\|\|` | `@operator` |
| `!` | `@operator` |
| `&`, `\|` (label expr) | `@operator` |
| `..` | `@operator` |

---

## Locals Model (locals.scm)

| Scope Unit | Capture | Reasoning |
|------------|---------|-----------|
| `statement` | `@local.scope` | Primary query scope |
| `union_statement` | `@local.scope` | Each branch is a separate scope |
| `node_pattern`.`variable` → `identifier` | `@local.definition` | MATCH (n:…) — `n` defined here |
| `relationship_body`.`variable` → `identifier` | `@local.definition` | -[r:…]- — `r` defined here |
| `path_pattern`.`variable` → `identifier` | `@local.definition` | p = (…) — `p` defined here |
| `return_item`.`alias` → `_symbolic_name` | `@local.definition` | … AS alias — alias defined |
| `yield_item`.`alias` → `identifier` | `@local.definition` | YIELD x AS y — y defined |
| `unwind_clause` → last `_symbolic_name` | `@local.definition` | UNWIND list AS item — item defined |
| `list_comprehension` → `identifier` | `@local.definition` | [x IN list | …] — x defined |
| `all_expression`/`any_expression`/`none_expression`/`single_expression` → `identifier` | `@local.definition` | ALL(x IN … WHERE …) |
| `reduce_expression`.`accumulator` → `identifier` | `@local.definition` | REDUCE(acc = 0, …) |
| `reduce_expression` → iterator `identifier` | `@local.definition` | REDUCE(…, item IN list) |
| `identifier` | (all other positions) | `@local.reference` | Use of a previously defined name |

---

## Tags Model (tags.scm)

| AST Subtree | Tag Capture | Name Capture | Notes |
|-------------|-------------|--------------|-------|
| `call_clause` → `name` → `procedure_name` | `@definition.function` | `@name` | CALL proc.name() |
| `function_call` → `name` → `function_name` | `@definition.function` | `@name` | func(args) |
