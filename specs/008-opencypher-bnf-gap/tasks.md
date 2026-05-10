# Tasks: openCypher BNF Grammar Coverage

**Input**: Design documents from `specs/008-opencypher-bnf-gap/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Organization**: Tasks are grouped by implementation slice, each mapping to a user story.
Each slice is independently testable: grammar rule → `tree-sitter generate` → corpus tests → validate.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different test bodies, same file is fine for corpus text additions)
- **[US#]**: Which user story this task belongs to
- Each slice ends with `tree-sitter test` to gate progress

---

## Phase 1: Baseline Verification

**Purpose**: Confirm the starting state is clean before any grammar changes

- [ ] T001 Run `tree-sitter test` and confirm all existing corpus tests pass (zero failures)
- [ ] T002 Run `tree-sitter generate` and confirm zero shift/reduce and reduce/reduce conflicts

**Checkpoint**: Green baseline — all slices can now proceed

---

## Phase 2: Slice 1 — `shortestPath` / `allShortestPaths` (Priority: P1) 🎯 MVP

**Goal**: `shortestPath((a)-[*]-(b))` and `allShortestPaths(...)` parse without ERROR nodes

**Independent Test**: `tree-sitter parse cypher/shortest_path.cypher` reports zero ERROR nodes

- [ ] T003 [US1] Add `legacy_shortest_path_pattern` rule to `grammar.js`: match `{ shortestpath | allshortestpaths } '(' node_pattern relationship_pattern node_pattern ')'`, add to `expression` choices, add `[$.legacy_shortest_path_pattern, $.function_call]` to `conflicts`
- [ ] T004 [US1] Run `tree-sitter generate` and verify zero conflicts after T003
- [ ] T005 [P] [US1] Add positive corpus tests for `shortestPath` and `allShortestPaths` in `test/corpus/patterns.txt` — cover: basic `shortestPath((a)-[*]-(b))`, path variable assignment `path = shortestPath(...)`, typed relationship `shortestPath((a)-[:KNOWS*]-(b))`, `allShortestPaths` variant
- [ ] T006 [P] [US1] Add negative corpus tests for malformed shortestPath in `test/corpus/patterns.txt` — cover: missing inner path, expression argument instead of path pattern
- [ ] T007 [US1] Run `tree-sitter test` and verify all tests pass (T005/T006 must pass)
- [ ] T008 [US1] Add `(legacy_shortest_path_pattern) @function` highlight capture to `queries/highlights.scm`
- [ ] T009 [US1] Run `tree-sitter parse cypher/shortest_path.cypher` and verify zero ERROR nodes (TCK gate for Slice 1)

**Checkpoint**: User Story 1 complete — `shortestPath` parses cleanly; lint tool reports zero ParseErrors on `cypher/shortest_path.cypher`

---

## Phase 3: Slice 2a — Inline WHERE in Patterns (Priority: P2)

**Goal**: `(n WHERE n.age > 30)` and `[r WHERE r.weight > 5]` parse without ERROR nodes

**Independent Test**: `tree-sitter parse` on a query with inline pattern predicates reports zero ERROR nodes

- [ ] T010 [US2] Add `optional($.where_clause)` as the last optional child (before `)`) in the `node_pattern` rule in `grammar.js`
- [ ] T011 [US2] Add `optional($.where_clause)` as the last component in all four branches of `relationship_body` in `grammar.js`
- [ ] T012 [US2] Run `tree-sitter generate` and verify zero conflicts after T010/T011
- [ ] T013 [P] [US2] Add positive corpus tests for inline WHERE in `test/corpus/patterns.txt` — cover: `(n WHERE n.active)`, `(n:Person WHERE n.age > 30)`, `(n {name: $name} WHERE n.active)`, `[r WHERE r.weight > 5]`, `[r:KNOWS WHERE r.since > 2020]`, combined node+rel inline WHERE
- [ ] T014 [P] [US2] Add negative corpus tests for inline WHERE in `test/corpus/patterns.txt` — cover: WHERE before label (wrong order), missing expression after WHERE inside pattern
- [ ] T015 [US2] Run `tree-sitter test` and verify all tests pass (T013/T014 must pass)

**Checkpoint**: User Story 2 complete — inline pattern predicates parse correctly

---

## Phase 4: Slice 2b — Map Projection (Priority: P2)

**Goal**: `n { .name, .age, score: 10, .* }` parses without ERROR nodes as a `map_projection` node

**Independent Test**: `RETURN n { .name }` parses with a `map_projection` node and no ERROR nodes

- [ ] T016 [US3] Add sub-rules to `grammar.js`: `field_selector` (`'.' identifier`), `all_fields_selector` (`'.' '*'`), `literal_map_field` (`identifier ':' expression`), `variable_selector` (`identifier`) — all with appropriate `field(...)` annotations per `data-model.md`
- [ ] T017 [US3] Add `map_projection_element` choice rule (union of T016 sub-rules) to `grammar.js`
- [ ] T018 [US3] Add `map_projection` rule to `grammar.js`: `prec.left(10, seq($.expression, '{', commaSep($.map_projection_element), '}'))`, add to `expression` choices, add `[$.map_projection, $.map_literal]` to `conflicts`
- [ ] T019 [US3] Run `tree-sitter generate` and verify zero conflicts after T016–T018
- [ ] T020 [P] [US3] Add positive corpus tests for map projection in `test/corpus/expressions.txt` — cover: `n { .name }`, `n { .name, .age }`, `n { .* }`, `n { .name, score: 10 }`, `n { .name, friend: m }`, nested `n { .name, friend: m { .name } }`
- [ ] T021 [P] [US3] Add negative corpus tests for map projection in `test/corpus/expressions.txt` — cover: `{ .name }` with no preceding expression (must parse as `map_literal`, not `map_projection`), empty projection `n {}`
- [ ] T022 [US3] Run `tree-sitter test` and verify all tests pass (T020/T021 must pass)
- [ ] T023 [US3] Add highlight captures for `(map_projection)`, `(field_selector)`, `(all_fields_selector)`, `(literal_map_field)` to `queries/highlights.scm`

**Checkpoint**: User Stories 1, 2, and 3 complete — all P1/P2 gaps resolved

---

## Phase 5: Slice 3a — GQL Path-Search Prefixes (Priority: P3)

**Goal**: `MATCH ANY SHORTEST (a)-[*]-(b)`, `MATCH SHORTEST 3 ...`, etc. parse correctly

**Independent Test**: `MATCH ALL (a:Person)-[*]-(b:Person) RETURN a, b` parses with `all_path_search` node and no ERROR nodes

- [ ] T024 [US4] Add `all_path_search` rule to `grammar.js`: `seq(kw('ALL'), optional(choice(kw('PATH'), kw('PATHS'))))`
- [ ] T025 [US4] Add `any_path_search` rule to `grammar.js`: `seq(kw('ANY'), optional($.integer_literal), optional(choice(kw('PATH'), kw('PATHS'))))`
- [ ] T026 [US4] Add `all_shortest_path_search`, `any_shortest_path_search` rules to `grammar.js`
- [ ] T027 [US4] Add `counted_shortest_path_search` and `counted_shortest_group_search` rules to `grammar.js` (with integer count and optional GROUP/GROUPS keyword)
- [ ] T028 [US4] Add `path_search_prefix` choice rule to `grammar.js` (union of T024–T027)
- [ ] T029 [US4] Update `match_clause` in `grammar.js` to accept `optional($.path_search_prefix)` immediately after the `MATCH`/`OPTIONAL MATCH` keywords, before `$.pattern`
- [ ] T030 [US4] Run `tree-sitter generate` and verify zero conflicts after T024–T029
- [ ] T031 [P] [US4] Add positive corpus tests for each path-search prefix form in `test/corpus/match_return.txt` — cover: `ALL`, `ANY`, `ANY 3`, `ALL SHORTEST`, `ANY SHORTEST`, `SHORTEST 3`, `SHORTEST 3 GROUPS`
- [ ] T032 [P] [US4] Add negative corpus tests for invalid path-search prefix syntax in `test/corpus/match_return.txt`
- [ ] T033 [US4] Run `tree-sitter test` and verify all tests pass (T031/T032 must pass)
- [ ] T034 [US4] Add highlight captures for `path_search_prefix` and sub-types in `queries/highlights.scm`

**Checkpoint**: User Story 4 complete — GQL path-search prefixes parse correctly

---

## Phase 6: Slice 3b — Quantified Path Patterns (Priority: P3)

**Goal**: `((a)-[:KNOWS]->(b)){1,3}` and `((a)-->(b))+` parse as `quantified_path_primary` nodes

**Independent Test**: `MATCH ((a)-[r]->(b)){2,5} RETURN a, b` parses with `quantified_path_primary` and no ERROR nodes

- [ ] T035 [US5] Add `fixed_quantifier` rule to `grammar.js`: `seq('{', field('count', $.integer_literal), '}')`
- [ ] T036 [US5] Add `general_quantifier` rule to `grammar.js`: `seq('{', optional(field('lower', $.integer_literal)), ',', optional(field('upper', $.integer_literal)), '}')`
- [ ] T037 [US5] Add `graph_pattern_quantifier` choice rule to `grammar.js`: `choice('+', '*', $.fixed_quantifier, $.general_quantifier)`
- [ ] T038 [US5] Add `quantified_path_primary` rule to `grammar.js`: parenthesized `node_pattern repeat1(seq(rel, node))` followed by `graph_pattern_quantifier`
- [ ] T039 [US5] Update `path_pattern` rule in `grammar.js` to allow `quantified_path_primary` nodes as path elements (extend `repeat(...)` body)
- [ ] T040 [US5] Run `tree-sitter generate` and verify zero conflicts after T035–T039
- [ ] T041 [P] [US5] Create `test/corpus/quantified_paths.txt` with positive corpus tests — cover: `{n,m}`, `{n}`, `{n,}`, `{,m}`, `+`, `*`, combinations in a longer path
- [ ] T042 [P] [US5] Add negative corpus tests to `test/corpus/quantified_paths.txt` — cover: quantifier on non-parenthesized path, `{-1}`, `{3,1}` (upper < lower)
- [ ] T043 [US5] Run `tree-sitter test` and verify all tests pass (T041/T042 must pass)
- [ ] T044 [US5] Add highlight captures for `quantified_path_primary`, `fixed_quantifier`, `general_quantifier` in `queries/highlights.scm`

**Checkpoint**: User Story 5 complete — quantified path patterns parse correctly

---

## Phase 7: Slice 3c — `YIELD … WHERE` (Priority: P3)

**Goal**: `CALL db.labels() YIELD label WHERE label STARTS WITH 'P'` parses with WHERE as child of `yield_clause`

**Independent Test**: The above query parses with a `where_clause` child in `yield_clause` and no ERROR nodes

- [ ] T045 [US6] Add `optional($.where_clause)` after the yield-item list in the `yield_clause` rule in `grammar.js`
- [ ] T046 [US6] Run `tree-sitter generate` and verify zero conflicts after T045
- [ ] T047 [P] [US6] Add positive corpus tests for `YIELD … WHERE` in `test/corpus/pipeline.txt` — cover: `YIELD item WHERE expr`, `YIELD * WHERE expr`, `YIELD item AS alias WHERE expr`, `YIELD item` without WHERE (must still pass)
- [ ] T048 [P] [US6] Add negative corpus tests for malformed YIELD in `test/corpus/pipeline.txt` — cover: WHERE before yield items (wrong order)
- [ ] T049 [US6] Run `tree-sitter test` and verify all tests pass (T047/T048 must pass)

**Checkpoint**: User Story 6 complete — all P3 user stories resolved

---

## Phase 8: Numeric Literal Extensions (Polish)

**Purpose**: Add INF/INFINITY/NAN literals and extend float/integer for suffixes and digit separators

- [ ] T050 Add `inf_literal` rule to `grammar.js`: `alias(token(/[Ii][Nn][Ff]/), 'inf_literal')` — listed in `expression` before `$.identifier`
- [ ] T051 Add `infinity_literal` and `nan_literal` rules to `grammar.js` analogously
- [ ] T052 Extend `float_literal` regex in `grammar.js` to accept optional `[fFdD]` suffix
- [ ] T053 Extend `integer_literal` and `float_literal` regexes in `grammar.js` to allow `_` digit separators between digit groups (e.g., `1_000_000`)
- [ ] T054 Run `tree-sitter generate` and verify zero conflicts after T050–T053
- [ ] T055 [P] Add positive corpus tests for INF/INFINITY/NAN in `test/corpus/literals.txt`
- [ ] T056 [P] Add positive corpus tests for float suffixes (`1.5f`, `2.0D`) and digit separators (`1_000`, `1_000.5`) in `test/corpus/literals.txt`
- [ ] T057 [P] Add negative corpus tests for invalid numeric forms in `test/corpus/literals.txt`
- [ ] T058 Run `tree-sitter test` and verify all tests pass (T055–T057 must pass)
- [ ] T059 Add `(inf_literal) @number`, `(infinity_literal) @number`, `(nan_literal) @number` captures to `queries/highlights.scm`

---

## Phase 9: Final Validation & Cleanup

**Purpose**: End-to-end verification across all slices

- [ ] T060 [P] Run `tree-sitter test` full suite one final time and confirm 100% pass rate (zero failures across all corpus files)
- [ ] T061 [P] Run `tree-sitter parse` on every file in `cypher/` and verify zero ERROR nodes in each
- [ ] T062 Run `cypher lint cypher/` and verify zero ParseErrors (only semantic lint warnings permitted)
- [ ] T063 Update `proposals/bnf-gap-analysis.md` notable gaps 1–7 to mark as resolved with the slice that addressed each

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1** (Baseline): No dependencies — start immediately
- **Phase 2** (Slice 1 / US1): Depends only on Phase 1 passing
- **Phases 3–7** (Slices 2a–3c): Each depends on Phase 1; each is independently additive to `grammar.js` so they can proceed in any order after baseline, but are shown in priority order
- **Phase 8** (Numeric): No blocking dependencies; can proceed any time after Phase 1
- **Phase 9** (Validation): Depends on all prior phases completing

### User Story Dependencies

- **US1 (P1)**: Independent after baseline — start here
- **US2 (P2)**: Independent from US1 (different node_pattern/relationship_body changes)
- **US3 (P2)**: Independent from US1/US2 (adds map_projection to expression)
- **US4 (P3)**: Independent from US1–US3 (adds prefix to match_clause)
- **US5 (P3)**: Independent; creates new corpus file
- **US6 (P3)**: Smallest slice; single `yield_clause` line change

### Within Each Slice

- Grammar rule task(s) → `tree-sitter generate` → corpus tests → `tree-sitter test` → highlights
- Positive and negative corpus tests [P] can be written in parallel
- `tree-sitter test` gates moving to the next slice

### Parallel Opportunities

- Within a slice: positive and negative corpus test tasks [P] can be written simultaneously
- After Phase 1: Slices 2a and 2b can be worked in parallel (they touch different rules)
- After Phase 1: Slices 3c and the numeric literal phase (Phase 8) can proceed in parallel with any other slice

---

## Parallel Example: Slice 2 (US2 + US3 in parallel)

```bash
# Parallel grammar edits (different rules, same file is fine sequentially):
Task T010: Add WHERE to node_pattern
Task T011: Add WHERE to relationship_body

# Then: tree-sitter generate (sequential)

# Parallel corpus test writing:
Task T013: Positive inline-WHERE corpus tests
Task T014: Negative inline-WHERE corpus tests

# US3 can start after baseline independently:
Task T016: Map projection sub-rules
Task T017: map_projection_element choice rule
Task T018: map_projection rule + conflict
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Baseline Verification (T001–T002)
2. Complete Phase 2: Slice 1 — `shortestPath` (T003–T009)
3. **STOP and VALIDATE**: `cypher lint cypher/` shows zero ParseErrors
4. Merge if stable — this alone eliminates all current false lint errors

### Incremental Delivery

1. Baseline → Slice 1 (`shortestPath`) → **MVP demo: zero ParseErrors**
2. Add Slice 2a (inline WHERE) → corpus tests green → merge
3. Add Slice 2b (map projection) → corpus tests green → merge
4. Add Slices 3a, 3b, 3c → corpus tests green → merge
5. Add numeric extensions → polish → final validation

### Parallel Strategy

With parallel capacity:
- Engineer A: Slices 1 and 2a (path patterns)
- Engineer B: Slice 2b (map projection) and 3b (quantified paths)
- Engineer C: Slices 3a (prefixes), 3c (YIELD), Phase 8 (numerics)

---

## Notes

- After every grammar.js change, run `tree-sitter generate` before running tests — tests against the generated parser, not grammar.js directly
- [P] tasks = written to different sections of the same corpus file or truly different files; no implementation dependency
- Constitution mandates ≥1 positive AND ≥1 negative corpus test per new rule — tasks T005/T006, T013/T014, etc. are paired to satisfy this
- `tree-sitter test` at the end of each slice is the dual-coverage gate check
- Corpus test format: use Tree-sitter s-expression syntax; node names must match exactly what `data-model.md` specifies
- Each `tree-sitter generate` step also produces updated `src/node-types.json` — the machine-readable version of `contracts/node-types.md`
