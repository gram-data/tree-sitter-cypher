---
description: "Task list for porting the openCypher BNF to a tree-sitter grammar"
---

# Tasks: Port openCypher BNF to Tree-sitter Grammar

**Input**: Design documents from `specs/001-port-opencypher-bnf/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Constitution gates per slice**: Fidelity (BNF anchor) · Dual-coverage (positive + negative corpus tests) · TCK (zero ERROR nodes on relevant TCK queries)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Directory layout and tool verification before any grammar work begins.

- [x] T001 Create `test/corpus/` directory and add `.gitkeep` placeholder
- [x] T002 [P] Create `queries/highlights.scm`, `queries/injections.scm`, `queries/locals.scm`, `queries/tags.scm` as empty files (required by Node binding loader in `bindings/node/index.js`)
- [x] T003 Run `tree-sitter generate` on the existing stub `grammar.js` to confirm toolchain works and `src/parser.c` is generated without error
- [x] T004 Run `npm install` to install `tree-sitter-cli` and `tree-sitter` dev dependencies from `package.json`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared helpers in `grammar.js` that every grammar rule depends on. MUST be complete before any user story begins.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T005 Add `kw(str)` case-insensitive keyword helper to `grammar.js` (see `specs/001-port-opencypher-bnf/research.md` Decision 1 for implementation)
- [x] T006 [P] Add `commaSep1(rule)` and `commaSep(rule)` list helpers to `grammar.js` (see `specs/001-port-opencypher-bnf/research.md` Decision 2)
- [x] T007 Add `extras` array to the grammar object in `grammar.js` to skip whitespace (`/\s+/`), line comments (`//...`), and block comments (`/* ... */`) (see research.md Decision 4)

**Checkpoint**: Run `tree-sitter generate` — must succeed. Foundation ready for all user stories.

---

## Phase 3: User Story 1 — Literals, Identifiers, Comments (Priority: P1) 🎯 MVP

**Goal**: Parse all atomic Cypher value types, identifiers, and comments into correctly typed leaf nodes with no ERROR nodes.

**Independent Test**: `tree-sitter test -f "literals"` passes all tests in `test/corpus/literals.txt`.

### Negative corpus tests for US1

- [x] T008 [P] [US1] Add negative corpus tests to `test/corpus/literals.txt`: malformed integer (`1_000`), unclosed string (`"hello`), invalid escape (`"\q"`), malformed hex (`0xGG`) — each must produce an ERROR node

### Positive corpus tests for US1

- [x] T009 [P] [US1] Add positive corpus tests to `test/corpus/literals.txt` covering: decimal integer, hex integer (`0x1A`), octal integer (`0o17`), float (`3.14`), scientific notation (`1.5e10`), single-quoted string, double-quoted string, string with escape sequences, `true`, `false`, `null`, unquoted identifier, backtick-escaped identifier, `$name` parameter, `$0` parameter

### Implementation for US1

- [x] T010 [US1] Add `integer_literal` rule to `grammar.js` covering decimal, hex (`0x`), and octal (`0o`) forms (BNF: `<unsigned decimal integer>`, `<unsigned hexadecimal integer>`)
- [x] T011 [US1] Add `float_literal` rule to `grammar.js` covering decimal and scientific notation forms (BNF: `<unsigned decimal in common notation>`, `<unsigned decimal in scientific notation>`)
- [x] T012 [P] [US1] Add `string_literal` rule to `grammar.js` covering single- and double-quoted strings with escape sequences (BNF: `<character string literal>`)
- [x] T013 [P] [US1] Add `boolean_literal` and `null_literal` rules to `grammar.js` using `kw()` helper (BNF: `<boolean literal>`, `<null literal>`)
- [x] T014 [P] [US1] Add `identifier` and `escaped_identifier` rules to `grammar.js` (BNF: `<regular identifier>`, `<escaped symbolic name>`)
- [x] T015 [US1] Add `parameter` rule to `grammar.js` for `$name` and `$0` forms (BNF: `<general parameter reference>`)
- [x] T016 [US1] Run `tree-sitter generate` then `tree-sitter test -f "literals"` — all tests must pass

**Checkpoint**: US1 independently functional. TCK gate: verify no ERROR nodes when parsing any literal expression from TCK feature files.

---

## Phase 4: User Story 2 — Minimal MATCH/RETURN (Priority: P2)

**Goal**: Parse the simplest complete Cypher query (`MATCH (n) RETURN n`) with correct clause structure in the tree.

**Independent Test**: `tree-sitter test -f "match_return"` passes all tests in `test/corpus/match_return.txt`.

### Negative corpus tests for US2

- [x] T017 [P] [US2] Add negative corpus tests to `test/corpus/match_return.txt`: missing RETURN keyword, missing closing paren in node pattern, empty MATCH with no pattern — each must produce an ERROR node

### Positive corpus tests for US2

- [x] T018 [P] [US2] Add positive corpus tests to `test/corpus/match_return.txt` covering: `MATCH (n) RETURN n`, `MATCH (n) RETURN n.name, n.age`, `MATCH (n) WHERE n.active = true RETURN n`, `OPTIONAL MATCH (n) RETURN n`, `MATCH (n) RETURN DISTINCT n`

### Implementation for US2

- [x] T019 [US2] Replace the stub `source_file` rule in `grammar.js` with `repeat1($.statement)` and add `statement` rule (BNF: `<statement block>`, `<statement>`)
- [x] T020 [US2] Add `match_clause` rule with `optional` OPTIONAL modifier, `kw('MATCH')`, pattern field, and optional `where_clause` field (BNF: `<simple match statement>`, `<optional match statement>`)
- [x] T021 [US2] Add minimal `node_pattern` rule — variable and closing parens only, no labels or properties yet (BNF: `<node pattern>` — partial, extended in US3)
- [x] T022 [US2] Add `where_clause` rule: `kw('WHERE')` followed by a placeholder `$.expression` rule (BNF: `<where clause>`)
- [x] T023 [US2] Add `return_clause`, `return_body`, and `return_item` rules including DISTINCT and AS alias (BNF: `<return statement>`, `<return item>`)
- [x] T024 [US2] Add placeholder `expression` rule as `choice` of `$.identifier`, `$.property_access`, and all literal types (extended fully in US4) (BNF: `<value expression>`)
- [x] T025 [US2] Add `property_access` rule: `prec.left(10, seq($.expression, '.', $.identifier))` (BNF: `<postfix expression>` with property name)
- [x] T026 [US2] Run `tree-sitter generate` then `tree-sitter test -f "match_return"` — all tests must pass

**Checkpoint**: US2 independently functional. A developer can parse `MATCH (n) RETURN n` and inspect the tree.

---

## Phase 5: User Story 3 — Graph Patterns (Priority: P3)

**Goal**: Parse all graph pattern shapes — node labels/properties, directed/undirected relationships, path variables, variable-length ranges, label expressions.

**Independent Test**: `tree-sitter test -f "patterns"` passes all tests in `test/corpus/patterns.txt`.

### Negative corpus tests for US3

- [ ] T027 [P] [US3] Add negative corpus tests to `test/corpus/patterns.txt`: unclosed node pattern `(n`, relationship without closing `]`, invalid range `[*-1]`, mismatched direction `<-[r]->` — each must produce an ERROR node

### Positive corpus tests for US3

- [ ] T028 [P] [US3] Add positive corpus tests to `test/corpus/patterns.txt` covering: `(n:Label)`, `(n:A&B)`, `(n {prop: 1})`, `(n:L {prop: 1})`, `(a)-[r]->(b)`, `(a)<-[r:TYPE]-(b)`, `(a)-[r:TYPE*1..3]->(b)`, `(a)-[*]->(b)`, `p = (a)-->(b)`, `(a)--(b)` (undirected), `(n IS Person)`

### Implementation for US3

- [ ] T029 [US3] Extend `node_pattern` rule in `grammar.js` to include optional `label` field (`$.label_expression`) and optional `properties` field (`$.property_map`) (BNF: `<node pattern>`)
- [ ] T030 [US3] Add `label_expression` rule with `|`, `&`, `!` operators and single `label_name`, using `prec` for operator precedence (BNF: `<label expression>`)
- [ ] T031 [US3] Add `property_map` and `property_key_value` rules for `{key: value}` inline maps (BNF: `<properties>`)
- [ ] T032 [US3] Add `relationship_pattern` rule covering direction (`-[...]->`  / `<-[...]-` / `-[...]-`), optional `relationship_body` (BNF: `<relationship pattern>`)
- [ ] T033 [US3] Add `relationship_body` rule: optional variable, optional label expression, optional `path_length`, optional property map (BNF: `<relationship detail>`)
- [ ] T034 [US3] Add `path_length` rule for `*`, `*n`, `*n..m` forms (BNF: `<path length>`)
- [ ] T035 [US3] Add `pattern`, `path_pattern`, and `path_variable` rules; update `match_clause` to use `$.pattern` (BNF: `<pattern>`, `<pattern element>`)
- [ ] T036 [US3] Run `tree-sitter generate` then `tree-sitter test -f "patterns"` — all tests must pass

**Checkpoint**: US3 independently functional. TCK gate: MATCH queries with patterns from TCK parse without ERROR nodes.

---

## Phase 6: User Story 4 — Expressions and WHERE (Priority: P4)

**Goal**: Parse all expression forms with correct operator precedence, including arithmetic, comparison, boolean, string predicates, function calls, list and map literals.

**Independent Test**: `tree-sitter test -f "expressions"` passes all tests in `test/corpus/expressions.txt`.

### Negative corpus tests for US4

- [ ] T037 [P] [US4] Add negative corpus tests to `test/corpus/expressions.txt`: unclosed function call `toUpper(`, unclosed list `[1, 2`, dangling operator `a +`, unmatched paren `(a + b` — each must produce an ERROR node

### Positive corpus tests for US4

- [ ] T038 [P] [US4] Add positive corpus tests to `test/corpus/expressions.txt` covering: `a + b * c` (precedence), `NOT x AND y OR z`, `n.age > 18`, `n.name = 'Alice'`, `n.name =~ '.*son'`, `n.name STARTS WITH 'A'`, `x IN [1,2,3]`, `n.val IS NULL`, `n.val IS NOT NULL`, `toUpper(n.name)`, `[1, 2, 3]`, `{key: 'val'}`, subscript `list[0]`, CASE expressions

### Implementation for US4

- [ ] T039 [US4] Replace the placeholder `expression` rule with the full `binary_expression` hierarchy using `prec.left` levels 1–7 (OR, XOR, AND, comparisons, add/sub, mul/div/mod) per `specs/001-port-opencypher-bnf/research.md` Decision 3 (BNF: `<boolean value expression>` through `<arithmetic term>`)
- [ ] T040 [US4] Add `unary_expression` for `NOT`, unary `-`, unary `+` using `prec.right` levels 4 and 9 (BNF: `<boolean factor>`, `<unary arithmetic>`)
- [ ] T041 [P] [US4] Add `is_null_expression`, `in_expression` rules (BNF: `<null predicate>`, `<in predicate>`)
- [ ] T042 [P] [US4] Add `starts_with_expression`, `ends_with_expression`, `contains_expression` rules (BNF: `<string operator expression>`)
- [ ] T043 [US4] Add `function_call` and `function_name` rules supporting simple and qualified names (e.g., `db.labels`) (BNF: `<function invocation>`)
- [ ] T044 [P] [US4] Add `list_literal` rule and `map_literal` rule (extending existing `property_map` for use as expressions) (BNF: `<list literal>`, `<map literal>`)
- [ ] T045 [US4] Add `subscript_expression` rule `prec.left(10, seq($.expression, '[', $.expression, ']'))` (BNF: `<subscript operator>`)
- [ ] T046 [US4] Run `tree-sitter generate` then `tree-sitter test -f "expressions"` — all tests must pass

**Checkpoint**: US4 independently functional. WHERE clauses in MATCH queries now fully parse with correct expression trees.

---

## Phase 7: User Story 5 — Data Mutation Clauses (Priority: P5)

**Goal**: Parse CREATE, SET, REMOVE, DELETE (including DETACH DELETE) with all set-item and remove-item forms.

**Independent Test**: `tree-sitter test -f "mutations"` passes all tests in `test/corpus/mutations.txt`.

### Negative corpus tests for US5

- [ ] T047 [P] [US5] Add negative corpus tests to `test/corpus/mutations.txt`: `CREATE` with no pattern, `SET` with no items, `DELETE` with no target, `DETACH` without `DELETE` — each must produce an ERROR node

### Positive corpus tests for US5

- [ ] T048 [P] [US5] Add positive corpus tests to `test/corpus/mutations.txt` covering: `CREATE (n:Person {name: 'Alice'})`, `SET n.age = 30`, `SET n += {active: true}`, `SET n = {name: 'Bob'}`, `SET n:Admin`, `REMOVE n:Temp`, `REMOVE n.prop`, `DELETE n`, `DETACH DELETE n`, `MATCH (n) CREATE (n)-[:KNOWS]->(m)`

### Implementation for US5

- [ ] T049 [US5] Add `create_clause` rule: `kw('CREATE')` followed by `$.pattern` (BNF: `<create statement>`)
- [ ] T050 [P] [US5] Add `set_clause`, `set_item` rules covering property assignment (`=`), map merge (`+=`), full replace (`=` with map), and label set (BNF: `<set statement>`, `<set item>`)
- [ ] T051 [P] [US5] Add `remove_clause`, `remove_item` rules for label removal and property removal (BNF: `<remove statement>`, `<remove item>`)
- [ ] T052 [US5] Add `delete_clause` rule with optional DETACH modifier (BNF: `<delete statement>`)
- [ ] T053 [US5] Update `statement` rule in `grammar.js` to include mutation clauses in the linear statement sequence (BNF: `<primitive data update statement>`)
- [ ] T054 [US5] Run `tree-sitter generate` then `tree-sitter test -f "mutations"` — all tests must pass

**Checkpoint**: US5 independently functional. TCK gate: CREATE/SET/DELETE queries from TCK parse without ERROR nodes.

---

## Phase 8: User Story 6 — Pipeline Clauses (Priority: P6)

**Goal**: Parse WITH, UNWIND, and the paging/ordering sub-clauses (ORDER BY, SKIP, LIMIT) so multi-step pipelines are fully structured.

**Independent Test**: `tree-sitter test -f "pipeline"` passes all tests in `test/corpus/pipeline.txt`.

### Negative corpus tests for US6

- [ ] T055 [P] [US6] Add negative corpus tests to `test/corpus/pipeline.txt`: `WITH` with no projection, `UNWIND` with no AS, `ORDER BY` with no expression, `SKIP` with no value — each must produce an ERROR node

### Positive corpus tests for US6

- [ ] T056 [P] [US6] Add positive corpus tests to `test/corpus/pipeline.txt` covering: `MATCH (n) WITH n RETURN n`, `MATCH (n) WITH n ORDER BY n.name ASC RETURN n`, `MATCH (n) WITH n SKIP 10 LIMIT 5 RETURN n`, `UNWIND [1,2,3] AS x RETURN x`, `MATCH (n) WITH n WHERE n.active = true RETURN n`, `WITH DISTINCT n RETURN n`

### Implementation for US6

- [ ] T057 [US6] Add `with_clause` rule: `kw('WITH')` + optional DISTINCT + `return_body` + optional `order_by_clause` + optional `skip_clause` + optional `limit_clause` + optional `where_clause` (BNF: `<with statement>`)
- [ ] T058 [US6] Add `unwind_clause` rule: `kw('UNWIND')` + expression + `kw('AS')` + identifier (BNF: `<unwind statement>`)
- [ ] T059 [P] [US6] Add `order_by_clause` and `sort_item` rules with optional ASC/DESC direction (BNF: `<order by clause>`)
- [ ] T060 [P] [US6] Add `skip_clause` and `limit_clause` rules (BNF: `<skip clause>`, `<limit clause>`)
- [ ] T061 [US6] Run `tree-sitter generate` then `tree-sitter test -f "pipeline"` — all tests must pass

**Checkpoint**: US6 independently functional. Multi-clause pipeline queries parse with correct stage structure.

---

## Phase 9: User Story 7 — MERGE and CALL (Priority: P7)

**Goal**: Parse MERGE with ON MATCH/ON CREATE actions and CALL for procedure invocation with optional YIELD.

**Independent Test**: `tree-sitter test -f "merge_call"` passes all tests in `test/corpus/merge_call.txt`.

### Negative corpus tests for US7

- [ ] T062 [P] [US7] Add negative corpus tests to `test/corpus/merge_call.txt`: `MERGE` with no pattern, `ON CREATE` without `SET`, `CALL` with unclosed argument list `CALL foo(`, `YIELD` without items — each must produce an ERROR node

### Positive corpus tests for US7

- [ ] T063 [P] [US7] Add positive corpus tests to `test/corpus/merge_call.txt` covering: `MERGE (n:Person {id: 1})`, `MERGE (n) ON CREATE SET n.ts = 0`, `MERGE (n) ON MATCH SET n.seen = true ON CREATE SET n.created = true`, `CALL db.labels()`, `CALL db.labels() YIELD label RETURN label`, standalone `CALL db.labels()` with no YIELD

### Implementation for US7

- [ ] T064 [US7] Add `merge_clause` rule: `kw('MERGE')` + pattern + `repeat($.merge_action)` (BNF: `<merge statement>`)
- [ ] T065 [US7] Add `merge_action` rule: `kw('ON')` + `choice(kw('MATCH'), kw('CREATE'))` + `$.set_clause` (BNF: `<merge action>`)
- [ ] T066 [US7] Add `call_clause` rule for in-query CALL with required argument list and optional `yield_clause` (BNF: `<named procedure call>`)
- [ ] T067 [P] [US7] Add `procedure_name` rule supporting simple and dot-qualified names (e.g., `db.labels`) (BNF: `<procedure reference>`)
- [ ] T068 [P] [US7] Add `yield_clause` and `yield_item` rules including AS alias (BNF: `<yield clause>`)
- [ ] T069 [US7] Update `source_file`/`statement` to accept standalone CALL (no YIELD required) (BNF: `<standalone procedure call>`)
- [ ] T070 [US7] Run `tree-sitter generate` then `tree-sitter test -f "merge_call"` — all tests must pass

**Checkpoint**: US7 independently functional. TCK gate: MERGE and CALL queries from TCK parse without ERROR nodes.

---

## Phase 10: User Story 8 — UNION and Advanced Expressions (Priority: P8)

**Goal**: Parse UNION/UNION ALL, CASE expressions, list comprehensions, pattern comprehensions, REDUCE, and existential quantifiers.

**Independent Test**: `tree-sitter test -f "union_advanced"` passes all tests in `test/corpus/union_advanced.txt`.

### Negative corpus tests for US8

- [ ] T071 [P] [US8] Add negative corpus tests to `test/corpus/union_advanced.txt`: `UNION` with no right-hand statement, `CASE` without `END`, list comprehension with no `IN`, `REDUCE` with no accumulator — each must produce an ERROR node

### Positive corpus tests for US8

- [ ] T072 [P] [US8] Add positive corpus tests to `test/corpus/union_advanced.txt` covering: `MATCH (n:A) RETURN n UNION MATCH (n:B) RETURN n`, `UNION ALL`, simple CASE, searched CASE, `[x IN list WHERE x > 0 | x*2]`, `[(n)-->(m) | m.name]`, `reduce(s=0, x IN xs | s+x)`, `ALL(x IN xs WHERE x > 0)`, `ANY(...)`, `NONE(...)`, `SINGLE(...)`, `count(*)`

### Implementation for US8

- [ ] T073 [US8] Add `union_statement` rule wrapping two statements with `kw('UNION')` + optional `kw('ALL')` (BNF: `<composite statement>`, `<composite conjunction>`)
- [ ] T074 [US8] Update `source_file` to use `choice($.union_statement, repeat1($.statement))` (BNF: `<program>`)
- [ ] T075 [P] [US8] Add `case_expression`, `case_when_clause`, `case_else_clause` rules for both simple and searched CASE forms (BNF: `<case expression>`)
- [ ] T076 [P] [US8] Add `list_comprehension` rule: `'[' identifier kw('IN') expression optional(where_clause) optional(seq('|', expression)) ']'` (BNF: `<list comprehension>`)
- [ ] T077 [P] [US8] Add `pattern_comprehension` rule for `[(pattern) | expression]` path value form (BNF: `<pattern comprehension>`)
- [ ] T078 [P] [US8] Add `reduce_expression` rule (BNF: `<reduce expression>`)
- [ ] T079 [P] [US8] Add `all_expression`, `any_expression`, `none_expression`, `single_expression` quantifier rules (BNF: `<all predicate>`, `<any predicate>`, etc.)
- [ ] T080 [US8] Add `count_star` rule for `count(*)` (BNF: `<count star>`)
- [ ] T081 [US8] Run `tree-sitter generate` then `tree-sitter test -f "union_advanced"` — all tests must pass

**Checkpoint**: US8 complete. Full openCypher BNF coverage achieved.

---

## Phase 11: Polish & Cross-Cutting Concerns

**Purpose**: TCK grammar-conformance pass, highlights query, and final quality gate check.

**Note on TCK scope**: The openCypher TCK tests both grammar and database execution. We only care about grammar conformance — that `tree-sitter parse` produces no ERROR nodes for the Cypher queries in the TCK. Database setup steps, result assertions, and execution semantics are ignored.

- [ ] T082a Write `scripts/extract-tck-queries.sh` that extracts all Cypher snippets from `When executing query:` triple-quote blocks in `references/openCypher/tck/features/**/*.feature` files and writes each snippet as a numbered `.cypher` file to `/tmp/tck-queries/`. Use awk to match lines between the `"""` open/close markers that follow `When executing query:`. Ignore `When executing control query:` (database setup) and all `Then`/`And` assertions (execution semantics).
- [ ] T082b Run `bash scripts/extract-tck-queries.sh` then run `tree-sitter parse` on every file in `/tmp/tck-queries/` and count lines containing `ERROR`. Assert count is zero — this is the constitution TCK gate. Include the total query count and ERROR count in the PR description.
- [ ] T083 Review `src/node-types.json` (generated) against `specs/001-port-opencypher-bnf/data-model.md` and confirm all 45 named node types are present; also walk all 12 top-level sections of `references/openCypher/grammar/openCypher.bnf` and verify each section has at least one corresponding named grammar rule (document as a comment in `grammar.js`)
- [ ] T084 [P] Add initial syntax highlighting rules to `queries/highlights.scm` for keyword, literal, identifier, and comment node types
- [ ] T085 Run `npm test` to confirm the Node.js binding smoke test (`bindings/node/binding_test.js`) passes
- [ ] T086 [P] Benchmark parse performance: run `time tree-sitter parse` on a 100-line Cypher file assembled from corpus test snippets and confirm wall time < 50ms; use the Node.js binding to call `tree.edit()` + `parser.parse()` on a single-line change and confirm elapsed time < 5ms; document both measurements as a comment on this task

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion — **BLOCKS** all user stories
- **US1 (Phase 3)**: Depends on Phase 2 only
- **US2 (Phase 4)**: Depends on US1 (uses literal/identifier rules)
- **US3 (Phase 5)**: Depends on US2 (extends node_pattern; uses property_map in expressions)
- **US4 (Phase 6)**: Depends on US3 (label_expression appears in expressions; property_access already in place)
- **US5 (Phase 7)**: Depends on US3 (CREATE uses pattern) and US4 (SET uses expressions)
- **US6 (Phase 8)**: Depends on US4 (WITH/ORDER BY use expressions)
- **US7 (Phase 9)**: Depends on US3 (MERGE uses pattern) and US4 (CALL uses expressions)
- **US8 (Phase 10)**: Depends on all prior slices
- **Polish (Phase 11)**: Depends on all user stories complete

### Parallel Opportunities Within Phases

Each phase has tasks marked `[P]` that can run concurrently:

- Phase 3 (US1): T008 (negative tests) ‖ T009 (positive tests); T012 ‖ T013 ‖ T014
- Phase 5 (US3): T027 ‖ T028; T030 ‖ T031; T034 ‖ T035
- Phase 6 (US4): T037 ‖ T038; T041 ‖ T042 ‖ T044 ‖ T045
- Phase 8 (US6): T055 ‖ T056; T059 ‖ T060
- Phase 10 (US8): T071 ‖ T072; T075 ‖ T076 ‖ T077 ‖ T078 ‖ T079
- Phase 11 (Polish): T082b depends on T082a; T083 ‖ T084 ‖ T085 ‖ T086 can run concurrently after T082b

---

## Implementation Strategy

### MVP First (US1 + US2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational helpers
3. Complete Phase 3: US1 (literals, identifiers)
4. Complete Phase 4: US2 (MATCH/RETURN)
5. **STOP and VALIDATE**: Parse `MATCH (n) RETURN n` — inspect tree
6. Any editor integration can begin at this point

### Incremental Delivery

Each slice (US1→US8) independently passes `tree-sitter test` and validates against relevant TCK queries before the next begins. No slice is started until the prior slice's checkpoint is verified.

---

## Notes

- `[P]` tasks = different files or independent sections, no shared state
- `[USn]` label maps each task to its spec user story for traceability
- Every slice MUST satisfy all three constitution gates before moving on
- `tree-sitter generate` MUST be run after every `grammar.js` change
- Never hand-edit files in `src/` — they are generated
