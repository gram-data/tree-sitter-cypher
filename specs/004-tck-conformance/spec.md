# Feature Specification: TCK Conformance — Complete openCypher Grammar Coverage

**Feature Branch**: `004-tck-conformance`  
**Created**: 2026-05-07  
**Status**: Draft  
**Input**: User description: "Complete TCK conformance with openCypher as described in @proposals/expanded-language-coverage.md"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Subquery Predicate Parsing (Priority: P1)

As a developer embedding Cypher queries in a tool or editor, I need `WHERE exists { MATCH ... }` subquery
predicates to parse without ERROR nodes so that tools based on this grammar work correctly with real-world
Neo4j queries.

**Why this priority**: The `exists { }` subquery predicate appears in ~10 TCK queries and is a commonly
used Neo4j Cypher construct. It currently produces ERROR nodes, blocking any downstream tooling that
depends on clean parse trees.

**Independent Test**: Parse any `.cypher` file containing `WHERE exists { MATCH (n)-->(m) }` and confirm
zero ERROR nodes in the resulting tree.

**Acceptance Scenarios**:

1. **Given** a Cypher query with `WHERE exists { MATCH (n)-[:REL]->(m) }`, **When** parsed, **Then** the tree contains an `exists_subquery` node with no ERROR children
2. **Given** a Cypher query with a nested `exists` predicate, **When** parsed, **Then** the entire subquery is correctly bounded
3. **Given** a malformed `WHERE exists { MATCH (n)` (unclosed brace), **When** parsed, **Then** an ERROR node appears at the expected location

---

### User Story 2 - Label Predicate as Boolean Expression (Priority: P2)

As a developer writing queries that check node labels inside expressions, I need `WHERE n:Person` (used as
a boolean expression, not as a pattern label) to parse correctly so label-based filtering queries produce
clean ASTs.

**Why this priority**: Label predicates in WHERE clauses are idiomatic Cypher. They affect how labels
can be checked and combined with other boolean expressions, and their absence causes parse errors on
many real queries.

**Independent Test**: Parse `MATCH (n) WHERE n:Person AND n.active = true RETURN n` and confirm the
label predicate is a child of a boolean expression node with no ERRORs.

**Acceptance Scenarios**:

1. **Given** `WHERE n:Person`, **When** parsed, **Then** a `label_predicate` node appears inside the expression with no ERROR
2. **Given** `WHERE n:Person AND n.active = true`, **When** parsed, **Then** the label predicate is combined with a boolean expression correctly
3. **Given** `WHERE n:A|B`, **When** parsed, **Then** the disjunction form of the label predicate is a named node with no ERROR

---

### User Story 3 - FOREACH Clause (Priority: P3)

As a developer running update queries over collections, I need `FOREACH (x IN list | SET x.active = true)`
to parse correctly so pipeline queries with bulk updates produce clean ASTs.

**Why this priority**: FOREACH is the standard Cypher mechanism for applying mutations to each element
in a list. Without it, update pipelines that process multiple nodes are unrepresentable in the grammar.

**Independent Test**: Parse `FOREACH (x IN [1,2,3] | SET x.val = 0)` and confirm a `foreach_clause`
node with no ERROR children.

**Acceptance Scenarios**:

1. **Given** `FOREACH (x IN list | SET x.active = true)`, **When** parsed, **Then** a `foreach_clause` node appears with no ERRORs
2. **Given** `FOREACH (x IN list | CREATE (x)-[:KNOWS]->(y))`, **When** parsed, **Then** the inner mutation clause is correctly nested
3. **Given** `FOREACH (x IN list |)` (empty body), **When** parsed, **Then** an ERROR node appears at the empty body position

---

### User Story 4 - Pattern Predicate in WHERE (Priority: P4)

As a developer writing existence checks in WHERE clauses, I need `WHERE (n)-->(m)` (a graph pattern
used directly as a boolean predicate) to parse correctly so path-based filtering queries produce clean ASTs.

**Why this priority**: Pattern predicates are the idiomatic way to express existence of a relationship
without binding it to a variable. They are distinct from MATCH patterns and require explicit grammar
support.

**Independent Test**: Parse `MATCH (n) WHERE (n)-[:KNOWS]->() RETURN n` and confirm the pattern predicate
is correctly parsed with no ERRORs.

**Acceptance Scenarios**:

1. **Given** `WHERE (n)-->(m)`, **When** parsed, **Then** a `pattern_predicate` node appears inside the WHERE expression
2. **Given** `WHERE (n)-[:KNOWS]->()`, **When** parsed, **Then** relationship type is preserved in the predicate node
3. **Given** `WHERE NOT (n)-->(m)`, **When** parsed, **Then** the pattern predicate is correctly negated as a boolean expression

---

### User Story 5 - Expanded Corpus Test Coverage (Priority: P5)

As a grammar contributor, I need the corpus test suite to grow from ~93 tests to ≥200 tests so that
edge cases and real-world query forms are validated beyond the minimal dual-coverage gate.

**Why this priority**: The current suite has exactly 2 negative tests per slice — enough to pass the
constitution gate but insufficient to catch edge cases. Expanded coverage validates error recovery,
operator precedence edge cases, and multi-clause pipeline forms that are missing from the current suite.

**Independent Test**: Run `tree-sitter test` and confirm all tests pass; confirm total test count ≥200.

**Acceptance Scenarios**:

1. **Given** the corpus test suite, **When** `tree-sitter test` runs, **Then** 100% of tests pass and total count ≥200
2. **Given** edge case inputs (chained UNION, multi-path MATCH, nested CASE), **When** parsed, **Then** each has a dedicated corpus test that passes
3. **Given** error-recovery inputs (mid-expression unexpected token, keyword used as identifier), **When** parsed, **Then** each has a dedicated negative corpus test confirming ERROR node placement

---

### Edge Cases

- What happens when `exists { }` contains multiple MATCH clauses?
- How does `n:A&B` label predicate interact with `AND`/`OR` boolean operators in WHERE?
- What happens when FOREACH body contains multiple mutation clauses?
- How does the parser recover after an error inside an `exists { }` block?
- Can pattern predicates be nested inside list comprehensions or CASE expressions?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The grammar MUST parse `WHERE exists { MATCH ... }` subquery predicates without ERROR nodes, anchored to the openCypher BNF `<exists expression>` production
- **FR-002**: The grammar MUST parse `n:Label` label predicates as boolean expressions within WHERE clauses, anchored to `<node labels predicate>`
- **FR-003**: The grammar MUST parse `FOREACH (var IN list | mutation)` as a clause in the query pipeline, anchored to `<foreach statement>`
- **FR-004**: The grammar MUST parse `(pattern)` graph patterns used directly as boolean predicates in WHERE clauses, anchored to `<pattern predicate>`
- **FR-005**: Every new grammar rule MUST have at least one positive and one negative corpus test (dual-coverage constitution gate)
- **FR-006**: All new grammar rules MUST trace to a named production in `references/openCypher/grammar/openCypher.bnf` (fidelity constitution gate)
- **FR-007**: After implementing all missing rules, `tree-sitter parse` on all TCK Cypher snippets MUST report zero ERROR nodes for non-template queries (TCK constitution gate)
- **FR-008**: The corpus test suite MUST grow to ≥200 tests, covering the gap categories documented in `proposals/expanded-language-coverage.md`
- **FR-009**: All 102 existing corpus tests MUST continue to pass (no regressions)

### Key Entities

- **Grammar rule**: A named production in `grammar.js` derived from a BNF production name (snake_case). Has a BNF anchor, corpus tests, and a TCK validation status.
- **Corpus test**: A positive or negative test case in `test/corpus/*.txt` using Tree-sitter's s-expression format. Has an input snippet and an expected syntax tree.
- **TCK query**: A Cypher snippet extracted from `references/openCypher/tck/features/**/*.feature` via `scripts/extract-tck-queries.sh`. Represents a real openCypher compatibility test input.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: TCK non-template pass rate reaches ≥98% — zero ERROR nodes when parsing any non-template TCK query (baseline: 80.9% / 1309 of 1617 total, of which ~218 are template placeholders, so target is 0 failures among the ~1399 real queries)
- **SC-002**: Corpus test count reaches ≥200 (baseline: 102 tests), with each new missing feature area having ≥3 positive tests and ≥3 negative tests
- **SC-003**: All 5 missing grammar features (`exists { }`, label predicate, FOREACH, pattern predicate, quantified path patterns) parse without ERROR nodes for any syntactically valid input
- **SC-004**: `tree-sitter test` passes 100% with no regressions against the 102 existing tests
- **SC-005**: Parse speed remains within current performance envelope — no measurable regression on the benchmark established in T086

## Assumptions

- Quantified path patterns (GQL-style `[*]->+`) are lower priority and may be deferred if they require significant grammar restructuring; the other 4 features are the primary target
- The ~218 TCK failures from template placeholders (`<pattern>`, `<temporal>`) are not real Cypher and are excluded from the TCK gate — they are counted separately and do not count against the pass rate
- The expanded corpus tests draw from the gap categories in `proposals/expanded-language-coverage.md` and from TCK feature files organized by clause/expression area
- Existing `grammar.js` helpers (`kw()`, `commaSep1()`, `prec` levels) are sufficient for the new rules; no new DSL primitives are needed
- The `scripts/extract-tck-queries.sh` script already exists and correctly extracts non-template, non-setup queries from TCK feature files
