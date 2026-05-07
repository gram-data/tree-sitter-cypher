---
description: "Task list for completing openCypher TCK conformance"
---

# Tasks: TCK Conformance — Complete openCypher Grammar Coverage

**Input**: Design documents from `specs/004-tck-conformance/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Constitution gates per slice**: Fidelity (BNF anchor) · Dual-coverage (positive + negative corpus tests) · TCK (zero ERROR nodes on relevant TCK queries)

---

## Phase 1: Setup (Baseline Verification)

**Purpose**: Establish the baseline parse state and confirm toolchain before any grammar changes.

- [X] T001 Run `bash scripts/extract-tck-queries.sh` to populate `/tmp/tck-queries/` with 1617 extracted Cypher snippets
- [X] T002 Run `tree-sitter parse /tmp/tck-queries/*.cypher 2>/dev/null | grep -c ERROR` and record the baseline ERROR count (expected: ~307 total = ~210 template + ~97 real)
- [X] T003 Run `tree-sitter test` and confirm all 102 existing tests pass with 100% success rate

**Checkpoint**: Baseline established. Any regression from 102 passing tests is a blocker.

---

## Phase 2: Foundational — Infrastructure Fixes (Blocking Prerequisites)

**Purpose**: Fix the `path_length` token regex and add bidirectional `<-->` relationship. These are self-contained token/pattern fixes that do not touch the `expression` rule, enabling safe parallel work on subsequent slices.

**⚠️ CRITICAL**: No user story grammar rules can be safely added until this phase is complete (avoids mid-stream conflicts).

### Negative corpus tests for path length

- [X] T004 [P] Add negative corpus tests to `test/corpus/patterns.txt` for invalid path length forms: `(a)-[:T*-1]->(b)` must produce ERROR at the `-1`; `(a)-[:T..2]->(b)` (no star) must produce ERROR

### Positive corpus tests for path length

- [X] T005 [P] Add positive corpus tests to `test/corpus/patterns.txt` covering the missing `*..N` forms: `(a)-[:T*..2]->(b)` (upper bound only), `(a)-[:T*..]->(b)` (explicit unbounded), `(a)-[:T*1..]->(b)` (lower bound only, already works — confirm)

### Implementation: path_length fix

- [X] T006 Replace the `path_length` token rule in `grammar.js` with:
  ```js
  path_length: _ => token(seq(
    '*',
    optional(choice(
      seq(/[0-9]+/, '..', /[0-9]*/),   // *N..M or *N..
      seq('..', /[0-9]*/),             // *..M or *..
      /[0-9]+/,                        // *N (exact)
    )),
  )),
  ```
  BNF: `<path length>` — adds `*..M` and `*..` forms. Run `tree-sitter generate` after the change.

### Bidirectional relationship `<-->`

- [X] T007 [P] Add negative corpus tests to `test/corpus/patterns.txt`: `MATCH (a)<->(b)` (missing dashes) must produce ERROR
- [X] T008 [P] Add positive corpus test to `test/corpus/patterns.txt`: `MATCH (a)<-->(b) RETURN a`, `MATCH (a)<-[r]->(b) RETURN r`
- [X] T009 Add `<-[optional body]->` as an explicit alternative in `relationship_pattern` in `grammar.js`. Place it BEFORE the existing undirected `-[]-` form so GLR prefers it:
  ```js
  seq('<-', optional(seq('[', optional($.relationship_body), ']')), '->'),
  ```
  BNF: documented Neo4j extension (see research.md Decision 8). Run `tree-sitter generate`.

### Checkpoint

- [X] T010 Run `tree-sitter generate` then `tree-sitter test` — all 102+ tests must pass. Run `tree-sitter parse` on a sample of the new positive test cases to confirm clean trees.

**Checkpoint**: Path length and bidirectional fixes complete. Foundation ready for expression-level rules.

---

## Phase 3: User Story 2 — Label Predicate Expression (Priority: P2) 🎯 Highest TCK Impact

**Goal**: Parse `n:Person` as a boolean expression in WHERE clauses, RETURN items, and binary boolean expressions. This is the single largest source of TCK failures (~34 queries).

**Independent Test**: `tree-sitter test -f "label predicate"` passes all tests in the label predicate section of `test/corpus/expressions.txt`.

### Negative corpus tests for US2

- [X] T011 [P] [US2] Add negative corpus tests to `test/corpus/expressions.txt`: `WHERE n:` (trailing colon with no label) must produce ERROR; `WHERE :Person` (label predicate with no subject) must produce ERROR

### Positive corpus tests for US2

- [X] T012 [P] [US2] Add positive corpus tests to `test/corpus/expressions.txt` covering:
  - `MATCH (a)-[:ADMIN]-(b) WHERE a:A RETURN a.id`
  - `MATCH (n) WHERE n:Person AND n.active = true RETURN n`
  - `MATCH (n) WHERE n IS Person RETURN n`
  - `MATCH (n) WHERE n:A&B RETURN n`
  - `MATCH (n) RETURN n:Foo AS result`
  - `MATCH (a) RETURN a, a:B AS result`
  - `MATCH (n:Single) OPTIONAL MATCH (n)-[r]-(m) WHERE m:NonExistent RETURN r`

### Implementation for US2

- [X] T013 [US2] Add `is_labeled_expression` rule to `grammar.js`:
  ```js
  // BNF: <is labeled predicate part 2> ::= <is label expression>
  is_labeled_expression: $ => prec.left(5, seq(
    $.expression,
    field('label', $.label_expression),
  )),
  ```
- [X] T014 [US2] Add `$.is_labeled_expression` to the `expression` choice list in `grammar.js`, placing it BEFORE `$.identifier` and `$.escaped_identifier` in the choice ordering so GLR prefers it when `:` follows an expression
- [X] T015 [US2] Add conflict declarations to the `conflicts` array in `grammar.js`:
  ```js
  [$.is_labeled_expression, $.set_item],    // SET n:Label vs is_labeled
  [$.is_labeled_expression, $.remove_item], // REMOVE n:Label vs is_labeled
  ```
- [X] T016 [US2] Run `tree-sitter generate` then `tree-sitter test` — all tests must pass (no regressions on SET/REMOVE corpus tests)

**Checkpoint**: US2 independently functional. `WHERE n:Person` and `RETURN n:Foo AS result` parse cleanly.

---

## Phase 4: User Story (Pattern Comprehension) — Priority: P2 parallel

**Goal**: Parse `[(n)-->() | n.name]` and `[p = (n)-->() | p]` — pattern comprehensions with and without path variable binding. Fixes ~12 TCK failures.

**Independent Test**: `tree-sitter test -f "pattern comprehension"` passes all tests in `test/corpus/union_advanced.txt`.

### Negative corpus tests for pattern comprehension

- [X] T017 [P] Add negative corpus tests to `test/corpus/union_advanced.txt`: `[(n)-->()]` (no `|` projection operator) must produce ERROR; `[(n) | n]` (no relationship in pattern) should prefer `list_comprehension` form — confirm it parses correctly as list_comprehension

### Positive corpus tests for pattern comprehension

- [X] T018 [P] Add positive corpus tests to `test/corpus/union_advanced.txt` covering:
  - `MATCH (n) RETURN [(n)-[:T]->(b) | b.name] AS list`
  - `MATCH (n) RETURN [p = (n)-->() | p] AS list`
  - `MATCH (a:A), (b:B) RETURN [p = (a)-->(b) | p] AS list`
  - `MATCH (n:A) RETURN [p = (n)-->(:B) | p] AS list`
  - `MATCH (n) WITH n, size([(n)-->() | 1]) AS deg RETURN deg`
  - `MATCH (n:X) RETURN size([(n)--() | 1]) > 0 AS b`

### Implementation for pattern comprehension

- [X] T019 Add `pattern_comprehension` rule to `grammar.js`:
  ```js
  // BNF: <pattern comprehension> ::= '[' <pattern source> <pattern filter and projection> ']'
  pattern_comprehension: $ => prec(3, seq(
    '[',
    optional(seq(field('variable', $.identifier), '=')),
    field('pattern', $.path_pattern),
    optional($.where_clause),
    '|',
    field('projection', $.expression),
    ']',
  )),
  ```
  Use `prec(3)` — higher than `list_comprehension` (`prec(2)`) so GLR prefers pattern_comprehension when `(` follows `[`.
- [X] T020 Add `$.pattern_comprehension` to the `expression` choice list in `grammar.js`
- [X] T021 Run `tree-sitter generate` then `tree-sitter test` — all tests must pass; confirm `list_comprehension` tests still pass (no regression)

**Checkpoint**: Pattern comprehension independently functional. Both `[(n)-->() | e]` and `[p = (n)-->() | p]` parse cleanly.

---

## Phase 5: User Story 1 — exists {} Subquery (Priority: P1)

**Goal**: Parse `WHERE exists { (n)-->() }` and `WHERE exists { MATCH (n)-->(m) RETURN true }`. Fixes ~10 TCK failures.

**Independent Test**: `tree-sitter test -f "exists"` passes all tests added to `test/corpus/union_advanced.txt`.

### Negative corpus tests for US1

- [X] T022 [P] [US1] Add negative corpus tests to `test/corpus/union_advanced.txt`:
  - `WHERE exists { }` (empty braces) must produce ERROR
  - `WHERE exists ( (n)-->() )` (parens instead of braces) must produce ERROR — this is parsed as `exists(...)` function call and then fails

### Positive corpus tests for US1

- [X] T023 [P] [US1] Add positive corpus tests to `test/corpus/union_advanced.txt` covering:
  - `MATCH (n) WHERE exists { (n)-->() } RETURN n`
  - `MATCH (n) WHERE exists { (n)-[:NA]->() } RETURN n`
  - `MATCH (n) WHERE exists { (n)-->(m) WHERE n.prop = m.prop } RETURN n`
  - `MATCH (n) WHERE exists { MATCH (n)-->() RETURN true } RETURN n`
  - `MATCH (n) WHERE exists { MATCH (n)-->(m) WITH n, count(*) AS c WHERE c = 3 RETURN true } RETURN n`
  - Nested: `MATCH (n) WHERE exists { MATCH (m) WHERE exists { (n)-[]->(m) } RETURN true } RETURN n`

### Implementation for US1

- [X] T024 [US1] Add `exists_expression` and `exists_subquery` rules to `grammar.js`:
  ```js
  // BNF: <exists expression> ::= EXISTS { <subquery expression argument> }
  exists_expression: $ => seq(
    kw('EXISTS'),
    '{',
    choice($.pattern, $.exists_subquery),
    '}',
  ),

  // BNF: <procedure specification> ::= <statement block>
  exists_subquery: $ => repeat1($.statement),
  ```
- [X] T025 [US1] Add `$.exists_expression` to the `expression` choice list in `grammar.js`
- [X] T026 [US1] Run `tree-sitter generate` then `tree-sitter test` — all tests must pass

**Checkpoint**: US1 independently functional. Both graph-pattern and multi-clause `exists { }` forms parse cleanly.

---

## Phase 6: User Story 4 — Pattern Predicate in WHERE (Priority: P4)

**Goal**: Parse `WHERE (n)-[]->()` and `WHERE NOT (n)-[:T]-()` as pattern predicates (path used as boolean expression). Fixes ~5 TCK failures.

**Independent Test**: `tree-sitter test -f "pattern predicate"` passes all tests added to `test/corpus/expressions.txt`.

### Negative corpus tests for US4

- [X] T027 [P] [US4] Add negative corpus tests to `test/corpus/expressions.txt`:
  - `WHERE (n)` alone (no relationship following) is parsed as parenthesized expression — confirm no ERROR, no `pattern_predicate` node
  - `WHERE (n)-` (relationship operator but no closing) must produce ERROR

### Positive corpus tests for US4

- [X] T028 [P] [US4] Add positive corpus tests to `test/corpus/expressions.txt` covering:
  - `MATCH (n) WHERE (n)-[]->() RETURN n`
  - `MATCH (n) WHERE (n)-[:REL1]-() RETURN n`
  - `MATCH (n) WHERE (n)<-[:REL1]-() RETURN n`
  - `MATCH (n) WHERE (n)-[:REL1*]->() RETURN n`
  - `MATCH (n) WHERE NOT (n)-->(m) RETURN n`
  - `MATCH (n), (m) WHERE (n)-[]->(m) RETURN n, m`
  - `MATCH (n), (m) WHERE (n)-[:REL1]->(m) RETURN n, m`
  - `MATCH (n) WHERE (n)-[:REL1]-() AND (n)-[:REL3]-() RETURN n`
  - `MATCH (n) WHERE (n)-[:REL1]-() OR (n)-[:REL2]-() RETURN n`

### Implementation for US4

- [X] T029 [US4] Add `pattern_predicate` rule to `grammar.js`:
  ```js
  // BNF: <pattern expression> ::= <simple path pattern>
  // Used as <boolean primary> alternative to <predicate>
  pattern_predicate: $ => prec.dynamic(2, seq(
    $.node_pattern,
    choice(
      $.relationship_pattern,
      seq($.relationship_pattern, $.node_pattern,
          repeat(seq($.relationship_pattern, $.node_pattern))),
    ),
  )),
  ```
- [X] T030 [US4] Add `[$.pattern_predicate, $.expression]` to the `conflicts` array in `grammar.js` — this resolves the `(n)` ambiguity between `node_pattern` (start of `pattern_predicate`) and `seq('(', $.expression, ')')`
- [X] T031 [US4] Add `$.pattern_predicate` to the `expression` choice list in `grammar.js`
- [X] T032 [US4] Run `tree-sitter generate` then `tree-sitter test` — all tests must pass; confirm `WHERE (expr)` parenthesized expressions still parse correctly (no regression)

**Checkpoint**: US4 independently functional. Pattern predicates in WHERE parse cleanly; parenthesized expressions are not affected.

---

## Phase 7: User Story 5 — Expanded Corpus Test Coverage (Priority: P5)

**Goal**: Grow total corpus test count from ~115+ (after Phases 2–6) to ≥200 tests. Cover the gap categories from `proposals/expanded-language-coverage.md`.

**Independent Test**: `tree-sitter test` passes 100% with total count ≥200.

### New corpus test file

- [X] T033 [P] Create `test/corpus/tck_edge_cases.txt` with the following section headers: `patterns advanced`, `expressions advanced`, `pipeline advanced`, `mutations advanced`, `union advanced extras`

### Patterns (add to tck_edge_cases.txt)

- [X] T034 [P] [US5] Add positive corpus tests for advanced pattern forms:
  - `MATCH (a), (b)--(c) RETURN a, b, c` (multiple comma-separated paths)
  - `MATCH (a)-->(b)-->(c)-->(d) RETURN d` (deeply chained)
  - `MATCH ()-->(n) RETURN n` (anonymous start node)
  - `MATCH (a)-[:KNOWS|LIKES]->(b) RETURN b` (label disjunction in rel)
  - `MATCH (a)-[r:T {since: 2020}]->(b) RETURN r` (property in relationship)
  - `MATCH p=(n)<-->(k)<-->(n) RETURN p` (chained bidirectional — after Slice B)

### Expressions (add to tck_edge_cases.txt)

- [X] T035 [P] [US5] Add positive corpus tests for expression edge cases:
  - `RETURN NOT a AND b` (NOT precedence)
  - `RETURN a + b * c + d` (arithmetic precedence)
  - `RETURN n.address.city` (chained property access)
  - `-n.age` (unary minus on property)
  - Nested CASE: `CASE WHEN x > 0 THEN CASE WHEN y > 0 THEN 1 ELSE 2 END ELSE 3 END`
  - Multiple WHEN clauses: `CASE x WHEN 1 THEN 'a' WHEN 2 THEN 'b' ELSE 'c' END`

### Expressions (add to tck_edge_cases.txt) — pattern in expression

- [X] T036 [P] [US5] Add positive corpus tests for pattern used in expression contexts (after Slice F):
  - `MATCH (n) RETURN (n)-[]->()` (pattern predicate in RETURN)
  - `MATCH (n) WITH (n)-[]->() AS x RETURN x` (pattern in WITH)
  - `MATCH (n) SET n.prop = head(nodes(head((n)-[:REL]->())))` (pattern in expression chain)

### Pipeline (add to tck_edge_cases.txt)

- [X] T037 [P] [US5] Add positive corpus tests for pipeline edge cases:
  - `MATCH (n) WITH n ORDER BY n.name ASC, n.age DESC RETURN n` (multi-column ORDER BY)
  - `MATCH (n) RETURN * ` (wildcard projection)
  - `UNWIND {name: 'Alice', age: 30} AS x RETURN x` (UNWIND a map)
  - Multiple consecutive WITH: `MATCH (n) WITH n WITH n RETURN n`

### Mutations (add to tck_edge_cases.txt)

- [X] T038 [P] [US5] Add positive corpus tests for mutation edge cases:
  - `SET n.a = 1, n.b = 2` (multiple SET items)
  - `SET n = {name: 'Alice'}` (full map replace)
  - `REMOVE n:A, n.prop` (multiple REMOVE items)
  - `DELETE n, m` (multiple DELETE targets)
  - `MERGE (a)-[r:FOO]->(b) RETURN r` (MERGE with relationship)

### UNION and advanced (add to tck_edge_cases.txt)

- [X] T039 [P] [US5] Add positive corpus tests for UNION and advanced:
  - `MATCH (n:A) RETURN n UNION MATCH (n:B) RETURN n UNION MATCH (n:C) RETURN n` (chained UNION)
  - `ANY(x IN xs WHERE x > 0)`, `NONE(x IN xs WHERE x > 0)`, `SINGLE(x IN xs WHERE x > 0)` (quantifiers)

### Negative tests (add to tck_edge_cases.txt)

- [X] T040 [US5] Add ≥3 negative tests per existing slice for error recovery quality:
  - `RETURN a + * b` → ERROR (mid-expression unexpected token)
  - `MATCH (n` → ERROR (unclosed node pattern mid-query)
  - `RETURN {name}` → ERROR (map key without value)
  - `WHERE` → ERROR (WHERE with no expression)

### Checkpoint

- [X] T041 Run `tree-sitter test` — must pass 100%; confirm total count ≥200

**Checkpoint**: US5 complete. ≥200 corpus tests, all passing.

---

## Phase 8: Polish — TCK Gate Validation

**Purpose**: Run the full TCK gate to confirm ≥98% non-template pass rate. Update highlights.scm for new node types. Run performance benchmarks.

- [X] T042a Re-run `bash scripts/extract-tck-queries.sh` to refresh `/tmp/tck-queries/` with current 1617 queries
- [X] T042b Run `tree-sitter parse /tmp/tck-queries/*.cypher 2>/dev/null | grep -c ERROR` — count must be ≤ ~221 (210 templates + ≤11 intentionally-invalid queries); document the exact count and breakdown in the PR description
- [X] T043 [P] Review `queries/highlights.scm` and add captures for new node types:
  - `(is_labeled_expression)` — treat the label as `@type`
  - `(exists_expression "EXISTS" @keyword)`
  - `(pattern_predicate)` — no new capture needed (uses existing node/rel captures)
  - `(pattern_comprehension "|" @operator)`
- [X] T044 [P] Run `npm test` to confirm Node.js binding smoke test still passes after all grammar changes
- [X] T045 [P] Run the T086 benchmark: `time tree-sitter parse` on the 100-line benchmark Cypher file; confirm wall time < 50ms. Document any performance change in a comment on this task.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1** (Setup): No dependencies — start immediately
- **Phase 2** (Foundational): Depends on Phase 1 — **BLOCKS** all user story grammar work
- **Phase 3** (US2 label predicate): Depends on Phase 2 only
- **Phase 4** (Pattern comprehension): Depends on Phase 2 only — runs in parallel with Phase 3
- **Phase 5** (US1 exists {}): Depends on Phase 2; benefits from Phase 3 being stable first (shares `expression` rule)
- **Phase 6** (US4 pattern predicate): Depends on Phases 3 and 4 complete (avoids conflict pile-up in `expression`)
- **Phase 7** (US5 corpus): Depends on Phases 3–6 (tests new rules); can begin T034–T040 in parallel with Phase 3–5 writing
- **Phase 8** (Polish): Depends on all phases complete

### Parallel Opportunities Within Phases

- Phase 2: T004 ‖ T005, T007 ‖ T008 (tests independent of each other)
- Phase 3: T011 ‖ T012 (negative ‖ positive tests)
- Phase 4: T017 ‖ T018 (negative ‖ positive tests)
- Phase 5: T022 ‖ T023 (negative ‖ positive tests)
- Phase 6: T027 ‖ T028 (negative ‖ positive tests)
- Phase 7: T033, T034, T035, T036, T037, T038, T039, T040 all parallelizable (different sections)
- Phase 8: T043 ‖ T044 ‖ T045 (different files)

---

## Implementation Strategy

### MVP First (Phase 2 + Phase 3 Only)

1. Complete Phase 1: Baseline
2. Complete Phase 2: path_length fix + bidir relationship
3. Complete Phase 3: label predicate expression
4. **STOP and VALIDATE**: Run TCK parse, expect ~34 fewer failures (from ~97 to ~63 real failures)
5. If clean: proceed to Phases 4–6

### Incremental Delivery

Each slice (Phases 3→6) independently reduces TCK failures and passes `tree-sitter test` before the next begins. No slice is started until the prior slice's checkpoint is verified.

---

## Notes

- `[P]` tasks = different files or independent sections, no shared state
- `[US1]` etc. map tasks to spec user stories for traceability
- Every slice MUST satisfy all three constitution gates before moving on
- `tree-sitter generate` MUST be run after every `grammar.js` change
- Never hand-edit files in `src/` — they are generated
- FOREACH is explicitly deferred (not in openCypher BNF — see research.md Decision 7)
- Quantified path patterns (`[*]+`) are explicitly deferred (complex restructuring — separate spec)
