# Research: TCK Conformance — Complete openCypher Grammar Coverage

**Feature**: `004-tck-conformance` | **Date**: 2026-05-07

## TCK Failure Analysis

Running `bash scripts/extract-tck-queries.sh` and then `tree-sitter parse` on all 1617 extracted queries:

| Category | Count | Notes |
|----------|-------|-------|
| Template placeholders (`<pattern>`, `<sort>`, etc.) | 210 | Not real Cypher; excluded from gate |
| **Real grammar gaps** | **97** | Root-caused below |
| Intentionally invalid syntax (expected ERROR) | ~11 | Part of the 97; should stay ERROR |
| **Net new grammar work** | **~86** | Failures fixable by grammar changes |

### Real failures by root cause

| Root Cause | Failures | Priority |
|------------|----------|----------|
| Label predicate as boolean expression (`n:Person` in WHERE/RETURN) | 34+ | P1 |
| Pattern comprehension (path variable form `[p = (n)-->() | e]`) | 12+ | P2 |
| `exists { }` subquery predicate | 10 | P3 |
| Path length missing form `*..N` (no lower bound) | 9 | P4 |
| Pattern predicate (`(n)-->(m)` as boolean, in WHERE) | ~5 | P5 |
| Pattern comprehension (basic `[(n)-->() | e]` not implemented) | ~4 | P2 |
| Parameter in node/relationship patterns (`n $param`) | 4 | P6 |
| Bidirectional relationship `<-->` | 2 | P7 |
| Intentionally invalid syntax | ~11 | — (keep as ERROR) |

---

## Decision 1: Label predicate expression

**Problem**: `WHERE a:A` fails because `a:A` as a boolean expression is not in the `expression` rule. The token sequence `identifier ':' identifier` is ambiguous with `property_map` keys inside `{k: v}`.

**BNF anchor**: `<is labeled predicate part 2> ::= <is label expression>` (line 718 of openCypher.bnf), where `<is label expression> ::= { <colon> | IS } <label expression>`. This is an `<advanced comparison predicate part 2>`.

**Decision**: Add `is_labeled_expression` rule as `prec.left(5, seq($.expression, $.label_expression))`. The `label_expression` already starts with `:` or `IS`, which disambiguates from `property_map` (which requires `{` before the key). Add `is_labeled_expression` to the `expression` choice.

**Conflict**: `$.set_item` already has a conflict with `$.expression` for the `:` token (SET n:Label). The same conflict applies. Add `[$.is_labeled_expression, $.set_item]` and `[$.is_labeled_expression, $.remove_item]` to the `conflicts` array if needed.

**Alternatives considered**: Using a postfix operator on `advanced_comparison_predicand` — rejected because tree-sitter models expressions as a flat choice, and introducing a separate predicate hierarchy would require significant restructuring.

---

## Decision 2: Pattern comprehension

**Problem**: Queries like `[(n)-[:T]->(b) | b.name]` and `[p = (n)-->() | p]` fail. Pattern comprehension is not implemented at all in `grammar.js` (only `list_comprehension` exists).

**BNF anchor**: `<pattern comprehension> ::= '[' <pattern source> <pattern filter and projection> ']'` where `<pattern source> ::= [ <binding variable> <equals operator> ] <simple path pattern>`.

**Decision**: Add `pattern_comprehension` rule: `prec(3, seq('[', optional(seq(field('variable', $.identifier), '=')), field('pattern', $.path_pattern), optional($.where_clause), '|', field('projection', $.expression), ']'))`. Use `prec(3)` (higher than `list_comprehension`'s `prec(2)`) to prefer pattern comprehension when the content starts with `(`. Add to `expression` choice.

**Disambiguation**: `list_comprehension` starts with `[ identifier IN ...`; pattern comprehension starts with `[ (` or `[ identifier = (`. The `IN` keyword (or `=`) after the first token disambiguates the two. GLR handles the ambiguity if needed.

---

## Decision 3: `exists { }` subquery

**Problem**: `WHERE exists { (n)-->() }` and `WHERE exists { MATCH (n)-->() RETURN true }` fail.

**BNF anchor**: `<exists expression> ::= EXISTS <left brace> <subquery expression argument> <right brace>` where `<subquery expression argument> ::= <procedure specification> | <graph pattern>`.

**Decision**: Add `exists_expression` rule:
```js
exists_expression: $ => seq(
  kw('EXISTS'),
  '{',
  choice(
    $.pattern,            // graph pattern form: EXISTS { (n)-->(m) }
    $.exists_subquery,    // procedure spec form: EXISTS { MATCH ... RETURN ... }
  ),
  '}',
)
exists_subquery: $ => seq(repeat1($.statement))
```

The `{` `}` braces make this syntactically unambiguous with `map_literal` (which uses `{key: value}`), `property_map`, and `function_call`.

**Alternatives considered**: Using a single catch-all `repeat1($.statement)` for the content — accepted for the subquery form; using `$.pattern` for the graph-pattern form. The grammar detects which form applies by lookahead (`MATCH` vs `(`).

---

## Decision 4: Path length `*..N` form

**Problem**: `[:T*..2]` fails because the current `path_length` token regex `* optional(seq(/[0-9]+/, optional(seq('..', /[0-9]*/))))` does not handle the `*..N` case (no lower bound).

**BNF anchor**: `<path length> ::= <asterisk> [ <unsigned decimal integer> ] | <asterisk> <unsigned decimal integer> <range operator> [ <unsigned decimal integer> ]`. The BNF allows `*..<max>` and `<min>..` forms.

**Decision**: Replace the path_length token with:
```js
path_length: _ => token(seq(
  '*',
  optional(choice(
    seq(/[0-9]+/, '..', /[0-9]*/),   // *N..M or *N..
    seq('..', /[0-9]*/),             // *..M or *..
    /[0-9]+/,                        // *N
  )),
))
```
This handles all five forms: `*`, `*N`, `*N..M`, `*N..`, `*..M`, `*..`.

---

## Decision 5: Pattern predicate (`(n)-->(m)` in boolean context)

**Problem**: `WHERE (n)-[]->()` fails because a path pattern used as a boolean expression is not handled. The parenthesized path would conflict with `seq('(', $.expression, ')')` in the expression rule.

**BNF anchor**: `<boolean primary> ::= <pattern expression> | <predicate>` and `<pattern expression> ::= <simple path pattern>`.

**Decision**: Add `pattern_predicate` rule for a path-pattern used as a boolean expression:
```js
pattern_predicate: $ => prec.dynamic(2, seq(
  $.node_pattern,
  choice($.relationship_pattern, repeat1(seq($.relationship_pattern, $.node_pattern))),
)),
```
Add to the `expression` choice with `prec.dynamic` to prefer it over a parenthesized expression when a relationship operator follows. Add a `conflicts` entry: `[$.node_pattern, $.expression]`.

**Complexity note**: This introduces a genuine ambiguity — `(n)` is both a `node_pattern` and a parenthesized `expression`. GLR resolves this by keeping both parse states and committing when the next token (a relationship operator `-`, `<-`, `-->`) confirms which interpretation is correct. This is the correct approach for a non-deterministic context-free grammar.

---

## Decision 6: Parameter in node/relationship patterns

**Problem**: `(n $param)` fails even though `node_pattern` has `optional(field('properties', choice($.property_map, $.parameter)))`. The GLR parser does not choose `$.parameter` for the properties field.

**Root cause**: After parsing `identifier` (the variable), the parser sees `$param`. Since `$.parameter` starts with `$` and `$.property_map` starts with `{`, the parser should correctly prefer `$.parameter`. The bug is likely that the `properties` field alternative order puts `$.property_map` first, and GLR does not backtrack after committing to that path.

**Decision**: This is already structurally correct in the grammar but may need a conflict declaration: `[$.node_pattern, $.expression]` (to handle `(` ambiguity) may also help here. If the issue persists, swap the order to `choice($.parameter, $.property_map)` in both `node_pattern` and `relationship_body`.

---

## Decision 7: FOREACH clause

**Problem**: FOREACH is not in the openCypher BNF — it is a Neo4j-specific extension.

**Constitution**: "Every grammar rule MUST correspond to a named production in the openCypher BNF... Rules that deviate from or extend the BNF MUST be explicitly documented with the rationale."

**Decision**: Defer FOREACH to a separate feature. The TCK does not test FOREACH (confirmed: no FOREACH queries in the TCK failures). Adding it would require a documented constitution deviation and should be scoped separately.

**Alternatives considered**: Adding it in this feature with documentation — rejected because it would be the first constitution deviation and warrants its own spec/review.

---

## Decision 8: Bidirectional relationship `<-->`

**Problem**: `MATCH (a)<-->(b)` fails. The `<-->` pattern is not in `relationship_pattern`.

**BNF note**: The openCypher BNF uses undirected relationships as `(a)-[]-(b)` (no arrows). The `<-->` form is a Neo4j-specific syntax sometimes seen in TCK tests.

**Decision**: Add `<-->` as an explicit alternative in `relationship_pattern`. This is a minor extension; document it as a Neo4j-specific shorthand for the undirected-with-explicit-markers form.

---

## Decision 9: Expanded corpus tests

**Decision**: Add tests to `test/corpus/` for all gap categories in `proposals/expanded-language-coverage.md`. Target: ≥200 total tests (baseline: 102). Organize new tests in new files per feature area: `tck_subqueries.txt`, `tck_label_predicates.txt`, `tck_patterns.txt`, `tck_path_lengths.txt`.

---

## Excluded from this feature

- FOREACH (Neo4j extension, no BNF anchor, no TCK coverage)
- Quantified path patterns GQL-style (`[*]+`) — in BNF but complex restructuring required
- `DELETE n:Person` — intentionally invalid syntax; should produce ERROR
- Malformed literals, em dash, etc. — should produce ERROR (correct behavior)
