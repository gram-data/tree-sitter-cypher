# Proposal: Expanded Corpus Test Coverage

**Status**: Draft
**Context**: After initial grammar implementation (US1–US8, 93 corpus tests, 80.9% TCK pass rate)

## Current State

The initial implementation has 93 corpus tests — 77 positive and 16 negative (exactly 2 per slice).
Each of the 8 implementation slices has representative happy-path coverage and a minimal pair of
error cases to satisfy the constitution's dual-coverage gate.

| Slice | Positive | Negative | Total |
|---|---|---|---|
| Literals / identifiers | 19 | 2 | 21 |
| MATCH / RETURN | 5 | 2 | 7 |
| Graph patterns | 11 | 2 | 13 |
| Expressions & WHERE | 14 | 2 | 16 |
| Mutations (CREATE/SET/REMOVE/DELETE) | 9 | 2 | 11 |
| Pipeline (WITH/UNWIND/ORDER BY) | 6 | 2 | 8 |
| MERGE & CALL | 6 | 2 | 8 |
| UNION & advanced expressions | 7 | 2 | 9 |
| **Total** | **77** | **16** | **93** |

## Gaps

### Negative tests (highest priority)

Every slice has exactly 2 error cases — enough to satisfy the gate but not enough to validate error
recovery quality. The following categories are untested:

- Mid-expression unexpected token (e.g., `RETURN a + * b`)
- Unclosed brackets at various nesting depths
- Keyword used as identifier without quoting (e.g., `MATCH (match)`)
- Multiple syntax errors in the same query (recovery after first error)
- Empty input and whitespace-only input
- Queries that are syntactically valid but structurally nonsensical (e.g., bare `WHERE true`)

### Missing positive coverage

**Patterns:**
- Multiple comma-separated paths in one MATCH: `MATCH (a), (b)--(c)`
- Deeply chained paths: `(a)-->(b)-->(c)-->(d)`
- Anonymous nodes with no variable: `MATCH ()-->(n)`
- Label disjunction in relationships: `[:KNOWS|LIKES]`
- Property map in relationship patterns: `[r:T {since: 2020}]`

**Expressions:**
- Operator precedence edge cases: `NOT a AND b` vs `NOT (a AND b)`, `a + b * c + d`
- Chained property access: `n.address.city`
- Slice notation: `list[1..3]`, `list[1..]`, `list[..3]`
- Pattern comprehensions: `[(n)-->(m) | m.name]`
- ALL / ANY / NONE / SINGLE quantifiers (only ALL currently tested)
- REDUCE: `reduce(s = 0, x IN list | s + x)`
- String concatenation: `'a' || 'b'`
- Regex match: `n.name =~ '.*son'`
- Exponentiation: `x ^ 2`
- Unary plus/minus: `-n.age`, `+1`
- Nested CASE expressions
- CASE with multiple WHEN clauses

**Pipeline:**
- Multiple consecutive WITH clauses
- UNWIND of a map literal
- ORDER BY multiple columns with mixed ASC/DESC
- RETURN with ORDER BY + SKIP + LIMIT together
- RETURN * (wildcard projection)

**MERGE / CALL:**
- CALL inside a MATCH pipeline: `MATCH (n) CALL proc(n) YIELD x RETURN x`
- CALL YIELD * (wildcard yield)
- MERGE with relationship pattern

**UNION:**
- Chained UNION: `A UNION B UNION C`
- UNION ALL vs UNION distinctness (structural, not semantic)
- Multi-statement file with `;` separator

**Mutations:**
- SET with multiple items: `SET n.a = 1, n.b = 2`
- SET full map replace: `SET n = {name: 'Alice'}`
- REMOVE multiple items: `REMOVE n:A, n.prop`
- DELETE multiple nodes: `DELETE n, m`

### Grammar features not yet implemented

These require new grammar rules before tests can be written:

- **`exists { }` subquery predicate** — `WHERE exists { MATCH (n)-->(m) }` (10 TCK queries affected)
- **Label predicate in expression** — `WHERE n:Person` as a boolean expression
- **FOREACH** — `FOREACH (x IN list | SET x.active = true)`
- **Pattern predicate** — `WHERE (n)-->(m)` as a boolean condition in WHERE
- **Quantified path patterns** — `MATCH (n)-[*]->+(m)` (GQL-style)

## Suggested Expansion

A target of **200–250 corpus tests** would give high confidence in the grammar:

- Bring negative tests to **5+ per slice** (focus on error recovery quality)
- Add **3–5 tests per missing positive category** listed above
- Add a dedicated `test/corpus/edge_cases.txt` for boundary conditions
- Add tests extracted directly from the TCK for each grammar feature area

The TCK's `references/openCypher/tck/features/` directories are organized by feature area
(clauses/match, expressions, etc.) and can serve as a systematic source for additional test inputs.

## Relationship to TCK Pass Rate

The current 80.9% TCK pass rate (1309/1617 queries) breaks down as:
- ~218 failures: TCK template placeholders (`<pattern>`, `<temporal>`) — not real Cypher
- ~90 failures: Real Cypher that needs additional grammar features (subqueries, label predicates)
- ~1309 passes: Valid Cypher queries that parse without ERROR nodes

Expanding corpus tests will not directly improve the TCK pass rate, but will validate that
the grammar handles the full range of syntax within each already-implemented feature.
