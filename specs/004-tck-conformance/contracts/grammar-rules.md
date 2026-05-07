# Contract: Grammar Rules for TCK Conformance

**Feature**: `004-tck-conformance` | **Date**: 2026-05-07

This contract documents the BNF anchor, tree shape, and test gate for each new grammar rule.
It is the fidelity gate reference: every rule here must be verifiable against the openCypher BNF.

---

## New Rules

### `is_labeled_expression`

**BNF**: `<is labeled predicate part 2> ::= <is label expression>` (openCypher.bnf line ~718)
**Prec**: `prec.left(5, ...)` — same level as `is_null_expression`, `in_expression`

**Tree shape**:
```
(is_labeled_expression
  <expression>          ; the subject being tested
  <label_expression>)   ; starts with ':' or 'IS'
```

**Positive test**: `WHERE n:Person` parses the WHERE body as `(is_labeled_expression (identifier "n") (label_expression ...))`
**Negative test**: `WHERE n:` (no label after colon) produces ERROR

**Conflicts**: May need `[$.is_labeled_expression, $.set_item]` and `[$.is_labeled_expression, $.remove_item]` because SET/REMOVE also use `:Label` syntax in a different context.

---

### `pattern_comprehension`

**BNF**: `<pattern comprehension>` (openCypher.bnf, Map projection section)
**Prec**: `prec(3, ...)` — higher than `list_comprehension` (`prec(2)`)

**Tree shape**:
```
(pattern_comprehension
  [variable: (identifier)]    ; optional path variable binding
  pattern: (path_pattern ...)
  [where: (where_clause ...)]
  projection: <expression>)
```

**Positive tests**:
- `[(n)-->() | n.name]` — basic form
- `[p = (n)-->() | p]` — with path variable
- `[(n)-[:T]->(m) WHERE m.active | m.name]` — with WHERE filter

**Negative test**: `[(n)-->()]` (no `|` projection) produces ERROR

---

### `exists_expression`

**BNF**: `<exists expression> ::= EXISTS { <subquery expression argument> }` (openCypher.bnf ~line 824)

**Tree shapes**:
```
; Graph pattern form
(exists_expression
  (pattern ...))

; Multi-clause form
(exists_expression
  (exists_subquery
    (statement ...)+))
```

**Positive tests**:
- `WHERE exists { (n)-->() }` — graph pattern form
- `WHERE exists { MATCH (n)-->(m) RETURN true }` — multi-clause form
- `WHERE exists { (n)-[:T]->() WHERE n.active }` — with WHERE inside

**Negative test**: `WHERE exists ( (n)-->() )` (parens instead of braces) produces ERROR

---

### `exists_subquery`

**BNF**: `<procedure specification> ::= <statement block>`

**Tree shape**:
```
(exists_subquery
  (statement ...)+)
```

Inline rule — appears only as a child of `exists_expression`.

---

### `pattern_predicate`

**BNF**: `<pattern expression> ::= <simple path pattern>` (boolean primary alternate)

**Tree shape**:
```
(pattern_predicate
  (node_pattern ...)
  (relationship_pattern ...)
  [(node_pattern ...)]*)
```

**Positive tests**:
- `WHERE (n)-[]->()` — basic pattern predicate
- `WHERE (n)-[:REL1]-()` — with relationship type
- `WHERE (n)-[:REL1*]->()` — with variable-length
- `WHERE NOT (n)-->(m)` — negated pattern predicate

**Negative test**: `WHERE (n)` alone (no relationship, ambiguous with parenthesized expression) — the parser should prefer parenthesized expression for `(n)` alone; only with a following `-` should it become a pattern_predicate.

**Conflicts**: `[$.pattern_predicate, $.expression]` — `(n)` is both `node_pattern` (start of pattern_predicate) and `seq('(', $.expression, ')')`.

---

## Modified Rules

### `path_length` (token fix)

**BNF**: `<path length>` — extended to include `*..M` (no lower bound)

**Before**: `* optional(seq(/[0-9]+/, optional(seq('..', /[0-9]*/))))`  
**After**: `* optional(choice(/[0-9]+/, seq(optional(/[0-9]+/), '..', /[0-9]*/))`

**Positive tests** (new):
- `(a)-[:T*..2]->(b)` — upper bound only
- `(a)-[:T*..]->(b)` — explicit unbounded (edge case)

**Negative tests** (already exist): `[*-1]` produces ERROR

---

### `relationship_pattern` (extension)

**Existing forms**: `-->`, `<--`, `-[]-`, `-[]->`, `<-[]-`
**New form**: `<-[optional body]->` (the `<-->` bidirectional marker)

**Positive test**: `MATCH (a)<-->(b) RETURN a` parses without ERROR
**Negative test**: `MATCH (a)<->(b)` (no dashes) produces ERROR — this is not valid Cypher

---

## Constitution Gate Summary

| Rule | BNF Anchor | Positive Test | Negative Test |
|------|-----------|---------------|---------------|
| `is_labeled_expression` | `<is labeled predicate part 2>` | `WHERE n:Person` | `WHERE n:` |
| `pattern_comprehension` | `<pattern comprehension>` | `[(n)-->() \| n.name]` | `[(n)-->()]` |
| `exists_expression` | `<exists expression>` | `exists { (n)-->() }` | `exists ( ... )` |
| `exists_subquery` | `<procedure specification>` | `exists { MATCH ... }` | — (inline) |
| `pattern_predicate` | `<pattern expression>` | `WHERE (n)-[]->()` | — |
| `path_length` (fix) | `<path length>` | `[:T*..2]` | — |
| `relationship_pattern` (ext) | `<relationship pattern>` | `(a)<-->(b)` | `(a)<->(b)` |
