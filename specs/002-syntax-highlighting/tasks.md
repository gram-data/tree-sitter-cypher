# Tasks: Cypher Syntax Highlighting and Code Navigation

**Input**: Design documents from `specs/002-syntax-highlighting/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/highlight-captures.md ✅

**Organization**: Tasks are grouped by user story. Phase 2 is a blocking grammar prerequisite — no query file work can start until it is complete.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)

---

## Phase 1: Setup

**Purpose**: Verify toolchain and confirm current grammar state.

- [X] T001 Confirm `tree-sitter generate` succeeds and `tree-sitter test` passes from a clean state in `grammar.js`
- [X] T002 Run `tree-sitter query queries/highlights.scm` on a sample query to document the current (incomplete) capture output as a baseline

---

## Phase 2: Foundational — Grammar Keyword Exposure (BLOCKS all user stories)

**Purpose**: The current `kw()` helper uses `token(regex)` which produces **no AST nodes** for keywords. This phase modifies `grammar.js` to produce capturable anonymous nodes for every keyword, without changing the parse tree structure of any named rule.

**⚠️ CRITICAL**: No query file work can begin until T003–T005 are complete and all corpus tests pass.

- [X] T003 Update the `kw()` helper in `grammar.js` (lines 11–16) to wrap each regex token with `alias(..., str.toLowerCase())`, producing: `alias(token(new RegExp(...)), str.toLowerCase())` — this single change causes every `kw('MATCH')` call to emit an anonymous node of type `'match'` in the AST
- [X] T004 Run `tree-sitter generate` to regenerate `src/parser.c` and `src/node-types.json` after the `kw()` change in `grammar.js`
- [X] T005 Run `tree-sitter test` to confirm all existing corpus tests pass after the grammar change — the aliasing change should be transparent to named-rule s-expression patterns but must be verified; fix any test failures before proceeding

**Checkpoint**: Run `tree-sitter parse --xml /tmp/test.cypher` on `MATCH (n) RETURN n` and confirm that `match` and `return` anonymous nodes now appear as children of `match_clause` and `return_clause` respectively.

---

## Phase 3: User Story 1 — Syntax Highlighting (Priority: P1) 🎯 MVP

**Goal**: Deliver a complete `queries/highlights.scm` that highlights all Cypher token categories in any tree-sitter-enabled editor.

**Independent Test**: Run `tree-sitter query queries/highlights.scm` on `specs/002-syntax-highlighting/quickstart.md`'s sample query and verify every keyword, literal, label, parameter, operator, comment, and punctuation token receives an appropriate capture.

### Implementation for User Story 1

- [X] T006 [US1] Add clause keyword captures to `queries/highlights.scm`: group `"match"`, `"return"`, `"create"`, `"merge"`, `"delete"`, `"set"`, `"remove"`, `"with"`, `"unwind"`, `"call"`, `"yield"`, `"union"`, `"where"`, `"order"`, `"by"`, `"skip"`, `"offset"`, `"limit"`, `"on"`, `"case"`, `"when"`, `"then"`, `"else"`, `"end"`, `"as"` all under `@keyword`
- [X] T007 [P] [US1] Add keyword-operator captures to `queries/highlights.scm`: group `"and"`, `"or"`, `"not"`, `"xor"`, `"in"`, `"is"`, `"contains"`, `"starts"`, `"ends"`, `"all"`, `"any"`, `"none"`, `"single"`, `"reduce"` under `@keyword.operator`
- [X] T008 [P] [US1] Add keyword-control and keyword-modifier captures to `queries/highlights.scm`: `"optional"` and `"detach"` under `@keyword.control`; `"distinct"` under `@keyword.control`; `"asc"`, `"ascending"`, `"desc"`, `"descending"` under `@keyword.modifier`
- [X] T009 [P] [US1] Add comment capture to `queries/highlights.scm`: verify whether `(_comment)` or the anonymous token pattern captures single-line (`//`) and block (`/* */`) comments, and add the working pattern under `@comment`; test with a query containing both comment styles
- [X] T010 [US1] Reorder captures in `queries/highlights.scm` for correct specificity — move specific patterns (labels, function names, property keys) BEFORE the generic `(identifier) @variable` fallback, following the order documented in `specs/002-syntax-highlighting/research.md` section 5
- [X] T011 [US1] Add property-key captures to `queries/highlights.scm`: `(property_key_value (identifier) @property)` for map literal keys and `(property_access property: [(identifier) (escaped_identifier)] @property)` for property read access
- [X] T012 [P] [US1] Add operator captures to `queries/highlights.scm` for anonymous punctuation tokens: group `"->"`, `"<-"`, `"-"` (in relationship context), `"="`, `"<>"`, `"<"`, `">"`, `"<="`, `">="`, `"=~"`, `"+"`, `"*"`, `"/"`, `"%"`, `"^"`, `"+="`, `"||"`, `"!"`, `"&"`, `"|"`, `".."` under `@operator`
- [X] T013 [P] [US1] Add punctuation captures to `queries/highlights.scm`: `"("`, `")"`, `"["`, `"]"`, `"{"`, `"}"` under `@punctuation.bracket`; `","`, `";"`, `"."` under `@punctuation.delimiter`
- [X] T014 [P] [US1] Add `count_star` capture to `queries/highlights.scm`: `(count_star) @function` and `path_length` capture: `(path_length) @number`
- [X] T015 [US1] Validate the complete `queries/highlights.scm` using `tree-sitter query queries/highlights.scm` on all three sample files from `specs/002-syntax-highlighting/quickstart.md` and confirm: (a) no keyword goes uncaptured, (b) labels receive `@type` not `@variable`, (c) property positions receive `@property`, (d) parameters receive `@variable.parameter`

**Checkpoint**: User Story 1 is complete when `tree-sitter query queries/highlights.scm` on a representative Cypher query produces captures for all keyword, literal, type, function, variable, operator, and punctuation tokens — zero uncaptured tokens of interest.

---

## Phase 4: User Story 2 — Local Scope and Variable Tracking (Priority: P2)

**Goal**: Deliver a complete `queries/locals.scm` that enables "rename symbol" and semantic reference highlighting in locals-aware editors (Helix, Zed, nvim-treesitter with local variable support).

**Independent Test**: Run `tree-sitter query queries/locals.scm` on `MATCH (n:Person)-[:KNOWS]->(m:Person) WITH n AS person RETURN person` and verify: `n` and `m` in the MATCH pattern are `@local.definition`, `n` in `WITH n AS person` is `@local.reference`, `person` after `AS` is `@local.definition`, and `person` in `RETURN` is `@local.reference`.

### Implementation for User Story 2

- [X] T016 [US2] Create scope boundaries in `queries/locals.scm`: capture `(statement) @local.scope` and `(union_statement) @local.scope` to establish the outermost scope for each Cypher query
- [X] T017 [US2] Add variable definition captures to `queries/locals.scm` for graph pattern bindings: `(node_pattern variable: (identifier) @local.definition)`, `(relationship_body variable: (identifier) @local.definition)`, and `(path_pattern variable: (identifier) @local.definition)`
- [X] T018 [US2] Add variable definition captures to `queries/locals.scm` for clause-level bindings: `(return_item alias: [(identifier) (escaped_identifier)] @local.definition)`, `(yield_item alias: (identifier) @local.definition)`, and the `AS` binding in `unwind_clause` (last `[(identifier) (escaped_identifier)]` child)
- [X] T019 [US2] Add variable definition captures to `queries/locals.scm` for comprehension and quantifier bindings: `(list_comprehension (identifier) @local.definition)`, `(all_expression (identifier) @local.definition)`, `(any_expression (identifier) @local.definition)`, `(none_expression (identifier) @local.definition)`, `(single_expression (identifier) @local.definition)`, `(reduce_expression accumulator: (identifier) @local.definition)`, and the iterator `identifier` in `reduce_expression`
- [X] T020 [US2] Add generic reference capture to `queries/locals.scm`: `(identifier) @local.reference` as a fallback for all identifier uses not already matched as definitions — place AFTER all definition patterns to avoid over-capturing
- [X] T021 [US2] Validate `queries/locals.scm` using `tree-sitter query queries/locals.scm` on the independent test query above and on a multi-WITH chain (`MATCH (n) WITH n AS a WITH a AS b RETURN b`) — verify definition/reference assignments are consistent and no definition site is also marked as a reference

**Checkpoint**: User Story 2 is complete when `tree-sitter query queries/locals.scm` on representative queries correctly marks every variable introduction as `@local.definition` and every subsequent use as `@local.reference`.

---

## Phase 5: User Story 3 — Code Navigation Tags (Priority: P3)

**Goal**: Deliver a complete `queries/tags.scm` that emits navigable `@definition.function` + `@name` tags for procedure and function references in Cypher files.

**Independent Test**: Run `tree-sitter query queries/tags.scm` on a file containing `CALL apoc.load.json($url) YIELD value` and `RETURN toUpper(value.name)` — verify `apoc.load.json` (procedure_name) and `toUpper` (function_name) each produce a `@definition.function` capture paired with a `@name` capture.

### Implementation for User Story 3

- [X] T022 [US3] Implement `queries/tags.scm` with procedure name tags: `(call_clause name: (procedure_name) @name) @definition.function` to emit navigable tags for `CALL` statement procedure references
- [X] T023 [US3] Add function call tags to `queries/tags.scm`: `(function_call name: (function_name) @name) @definition.function` to emit navigable tags for function invocations
- [X] T024 [US3] Validate `queries/tags.scm` using `tree-sitter query queries/tags.scm` on the independent test query and confirm both `@definition.function` and `@name` captures appear for each procedure and function call; verify output is parseable by ctags-compatible tools

**Checkpoint**: User Story 3 is complete when `tree-sitter query queries/tags.scm` produces `@definition.function` + `@name` pairs for all procedure and function references in a representative Cypher file.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, constitution gates, and documentation.

- [X] T025 [P] Add a `test/corpus/highlights.txt` corpus file with at least 6 test cases exercising: keyword highlighting context, label vs variable disambiguation, parameter highlighting, property access, and comment parsing — each test uses a representative Cypher snippet and documents the expected s-expression shape of the annotated parse tree; include at least one negative test per keyword context (missing keyword → ERROR node at expected position)
- [X] T025b [P] Add graceful ERROR node handling to `queries/highlights.scm`: verify that absence of explicit ERROR captures does not cause query validation failures, and confirm that valid sub-trees within a partially-invalid query still receive correct captures (test with an intentionally malformed query such as `MATCH (n RETURN n`)
- [X] T026 Run `tree-sitter test` to confirm all corpus tests pass after Phase 2's grammar change and the new highlights corpus tests
- [X] T027 [P] Run `tree-sitter parse` on a representative sample of TCK queries from `references/openCypher/tck/` and confirm zero ERROR nodes — validates the constitution's TCK gate for the keyword aliasing grammar change
- [X] T028 Run the full quickstart validation from `specs/002-syntax-highlighting/quickstart.md` for all three query files end-to-end and confirm outputs match expected captures documented in `specs/002-syntax-highlighting/data-model.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — run immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — **BLOCKS all query file work**
- **Phase 3 (US1)**: Requires Phase 2 complete (keyword nodes must exist in AST)
- **Phase 4 (US2)**: Requires Phase 2 complete; independent of Phase 3 (different file: locals.scm)
- **Phase 5 (US3)**: Requires Phase 2 complete; independent of Phases 3–4 (different file: tags.scm)
- **Phase 6 (Polish)**: Requires all of Phases 3–5 complete

### User Story Dependencies

- **US1 (highlights.scm)**: Requires grammar keyword change (Phase 2) — no dependency on US2 or US3
- **US2 (locals.scm)**: Requires grammar keyword change (Phase 2) — independent of US1 and US3
- **US3 (tags.scm)**: Requires grammar keyword change (Phase 2) — independent of US1 and US2

### Within Each User Story

- US1: T006–T009 can run in parallel (separate capture groups); T010 (reordering) must come before T011–T013; T015 (validation) last
- US2: T016 (scopes) first; T017–T020 can run in parallel; T021 (validation) last
- US3: T022 and T023 can run in parallel; T024 (validation) last

### Parallel Opportunities

After Phase 2 completes, all three query files can be worked on simultaneously:
- US1 work: `queries/highlights.scm`
- US2 work: `queries/locals.scm`
- US3 work: `queries/tags.scm`

Within US1, tasks T007, T008, T009, T012, T013, T014 are all parallelizable (different capture groups, no file conflicts when editing different sections).

---

## Parallel Example: After Phase 2 Completes

```text
# All three user stories can launch in parallel:
Task (US1): "Expand queries/highlights.scm with keyword captures"
Task (US2): "Create queries/locals.scm with scope boundaries"
Task (US3): "Implement queries/tags.scm with procedure name tags"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Verify toolchain
2. Complete Phase 2: Grammar keyword exposure (CRITICAL)
3. Complete Phase 3: Expand `highlights.scm`
4. **STOP and VALIDATE**: Run `tree-sitter query queries/highlights.scm` on real Cypher queries
5. Deploy/share — highlighting is complete and independently useful

### Incremental Delivery

1. Phase 1 + 2 → Foundation ready (grammar exposes keywords)
2. Phase 3 → US1: Full syntax highlighting (MVP — most visible user value)
3. Phase 4 → US2: Add variable scoping for "rename symbol" support
4. Phase 5 → US3: Add code navigation tags for multi-file workflows
5. Phase 6 → Polish and TCK validation gate

---

## Notes

- `[P]` tasks write to different files or different sections — safe to run in parallel
- `[Story]` label maps each task to the user story for independent traceability
- Phase 2 is the only inter-story blocker; after it completes, all three stories are independent
- The grammar change in T003 is the only modification to `grammar.js` — all other tasks edit query files
- After T003, always run T004 (`tree-sitter generate`) before testing or querying
- The `kw()` helper change is a one-line modification with broad effect; read research.md section 1 before implementing T003
