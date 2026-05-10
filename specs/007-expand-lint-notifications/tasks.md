# Tasks: Expand Lint Coverage with Neo4j Notification Codes

**Input**: Design documents from `specs/007-expand-lint-notifications/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/rule-file-format.md

**Organization**: Tasks are grouped by user story. US1 and US2 are both P1 and can proceed in parallel once Foundational is complete.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: No new project structure is needed — this feature extends existing files in `tools/cypher/`.

- [x] T001 Verify `cargo test` passes on branch `007-expand-lint-notifications` before any changes (`tools/cypher/`)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Extend the `Rule` struct and `.scm` parser to carry an optional `code` field. All three rule stories depend on this because their `.scm` files include a `Code:` header and `make_diagnostic` must pass it through.

**⚠️ CRITICAL**: No user story rule can emit a `code`-bearing diagnostic until this phase is complete.

- [x] T002 Add `pub code: Option<String>` field to the `Rule` struct in `tools/cypher/src/rules.rs`
- [x] T003 Parse the optional `Code:` header line in `parse_rule_file()` in `tools/cypher/src/rules.rs` (e.g. `;; Code: 03N90` → `rule.code = Some("03N90".into())`)
- [x] T004 Update `make_diagnostic()` in `tools/cypher/src/lint.rs` to set `code: rule.code.clone()` instead of `code: None`
- [x] T005 [US5] Add integration test asserting that a rule `.scm` with a `Code:` header produces a diagnostic with a matching `code` field in `tools/cypher/tests/lint_integration.rs`
- [x] T006 [US5] Add integration test asserting that a rule `.scm` without a `Code:` header produces a diagnostic with `code: None` (omitted from JSON) in `tools/cypher/tests/lint_integration.rs`

**Checkpoint**: `cargo test` passes — `code` field flows from `.scm` header through `Rule` into `Diagnostic` JSON.

---

## Phase 3: User Story 1 — Cartesian Product Detection (Priority: P1) 🎯 MVP

**Goal**: Warn when a `MATCH` clause contains two or more disconnected `path_pattern` nodes, mirroring Neo4j notification 03N90.

**Independent Test**: `cypher lint -e "MATCH (a:User), (b:Order) RETURN a, b"` emits a `CartesianProduct` warning with `code: "03N90"`. A query with a single pattern emits nothing.

- [x] T007 [P] [US1] Create `tools/cypher/rules/structural/cartesian_product.scm` with `Rule: CartesianProduct`, `Severity: Warning`, `Applies-to: structural`, `Message:`, `Code: 03N90`, and the tree-sitter query `(match_clause pattern: (pattern (path_pattern) (path_pattern) @hit))`
- [x] T008 [US1] Register `cartesian_product.scm` in `builtin_rules()` in `tools/cypher/src/rules.rs` by adding it to the `structural_sources` slice
- [x] T009 [US1] Add integration test: `"MATCH (a:User), (b:Order) RETURN a, b"` → `CartesianProduct` warning at second pattern, `code == "03N90"` in `tools/cypher/tests/lint_integration.rs`
- [x] T010 [US1] Add integration test: `"MATCH (a:User)-[:PLACED]->(b:Order) RETURN a, b"` → no `CartesianProduct` diagnostic in `tools/cypher/tests/lint_integration.rs`
- [x] T011 [US1] Add integration test: three disconnected patterns produces two `CartesianProduct` warnings in `tools/cypher/tests/lint_integration.rs`

**Checkpoint**: `cargo test` and `cypher lint -e "MATCH (a), (b) RETURN a, b"` both produce the expected diagnostic.

---

## Phase 4: User Story 2 — Deprecated `id()` Function (Priority: P1)

**Goal**: Warn when the `id()` function is called (bare, unqualified), pointing users to `elementId()` instead, mirroring Neo4j notification 01N01.

**Independent Test**: `cypher lint -e "MATCH (n) RETURN id(n)"` emits a `DeprecatedFunction` warning with `code: "01N01"` and a message mentioning `elementId()`. `elementId(n)` emits nothing.

- [x] T012 [P] [US2] Create `tools/cypher/rules/structural/deprecated_id_function.scm` with `Rule: DeprecatedFunction`, `Severity: Warning`, `Applies-to: structural`, `Message: id() is deprecated in Neo4j 5. Use elementId() instead, which returns a stable string identifier.`, `Code: 01N01`, and the tree-sitter query matching a `function_call` whose `function_name` has exactly one `identifier` child equal to `"id"`
- [x] T013 [P] [US2] Register `deprecated_id_function.scm` in `builtin_rules()` in `tools/cypher/src/rules.rs`
- [x] T014 [US2] Add integration test: `"MATCH (n) RETURN id(n)"` → `DeprecatedFunction` warning, message contains `elementId`, `code == "01N01"` in `tools/cypher/tests/lint_integration.rs`
- [x] T015 [US2] Add integration test: `"MATCH (n) RETURN elementId(n)"` → no `DeprecatedFunction` diagnostic in `tools/cypher/tests/lint_integration.rs`
- [x] T016 [US2] Add integration test: `"MATCH (r:REL) WHERE id(r) > 0 RETURN r"` → `DeprecatedFunction` warning (fires on relationship too) in `tools/cypher/tests/lint_integration.rs`
- [x] T017 [US2] Add integration test: `"MATCH (n) RETURN apoc.id(n)"` → no `DeprecatedFunction` diagnostic (qualified name is not flagged) in `tools/cypher/tests/lint_integration.rs`

**Checkpoint**: `cargo test` passes and `cypher lint -e "RETURN id(1)"` emits the expected diagnostic.

---

## Phase 5: User Story 3 — Dynamic Property Access (Priority: P2)

**Goal**: Emit an information diagnostic when a subscript expression uses a variable or parameter as the property key (`n[$key]`, `n[variable]`), mirroring Neo4j notification 03N95.

**Independent Test**: `cypher lint -e "MATCH (n) WHERE n[\$key] IS NOT NULL RETURN n"` emits a `DynamicProperty` information diagnostic with `code: "03N95"`. `n.name` and `n[0]` and `n["name"]` emit nothing.

- [x] T018 [P] [US3] Create `tools/cypher/rules/structural/dynamic_property.scm` with `Rule: DynamicProperty`, `Severity: Information`, `Applies-to: structural`, `Message: Dynamic property key prevents index use. Consider using a static property name if the key is known at query-write time.`, `Code: 03N95`, and a tree-sitter query matching `subscript_expression` whose key expression is a `parameter` or `identifier`
- [x] T019 [P] [US3] Register `dynamic_property.scm` in `builtin_rules()` in `tools/cypher/src/rules.rs`
- [x] T020 [US3] Add integration test: `"MATCH (n) WHERE n[$key] IS NOT NULL RETURN n"` → `DynamicProperty` information, `code == "03N95"` in `tools/cypher/tests/lint_integration.rs`
- [x] T021 [US3] Add integration test: `"MATCH (n) RETURN n.name"` → no `DynamicProperty` diagnostic in `tools/cypher/tests/lint_integration.rs`
- [x] T022 [US3] Add integration test: `"MATCH (n) RETURN n[0]"` → no `DynamicProperty` diagnostic (integer literal key is static) in `tools/cypher/tests/lint_integration.rs`
- [x] T023 [US3] Add integration test: `"MATCH (n) RETURN n[\"name\"]"` → no `DynamicProperty` diagnostic (string literal key is static) in `tools/cypher/tests/lint_integration.rs`
- [x] T024 [US3] Add integration test: `"SET n[$key] = 1"` → `DynamicProperty` diagnostic fires on write side in `tools/cypher/tests/lint_integration.rs`

**Checkpoint**: `cargo test` passes and `cypher lint -e "MATCH (n) WHERE n[\$k] IS NOT NULL RETURN n"` emits the expected diagnostic.

> **US4 — `DeprecatedRelationshipTypeList` (deferred)**: `[:A|:B]` already produces a `ParseError`
> in the grammar (MISSING node). No tasks required; see `research.md` for full rationale.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: End-to-end validation and JSON contract verification.

- [x] T025 [P] [US5] Add `--json` integration test verifying `CartesianProduct` diagnostic contains `"code": "03N90"` in JSON output in `tools/cypher/tests/lint_integration.rs`
- [x] T026 [P] [US5] Add `--json` integration test verifying `DeprecatedFunction` diagnostic contains `"code": "01N01"` in JSON output in `tools/cypher/tests/lint_integration.rs`
- [x] T027 [P] [US5] Add `--json` integration test verifying `DynamicProperty` diagnostic contains `"code": "03N95"` in JSON output in `tools/cypher/tests/lint_integration.rs`
- [x] T028 Verify all three rules fire correctly from a markdown fenced `\`\`\`cypher` block via `cypher lint` on a `.md` file (manual smoke test using `quickstart.md` examples)
- [x] T029 Run full `cargo test` suite and confirm zero regressions against existing rules (`UnlabelledNode`, `UnboundedRelationship`, `MissingToolName`, etc.)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1** (T001): No dependencies — verify baseline immediately
- **Phase 2** (T002–T006): Depends on Phase 1. **BLOCKS** Phases 3–5
- **Phase 3** (T007–T011): Depends on Phase 2. US1.
- **Phase 4** (T012–T017): Depends on Phase 2. US2. Can run in parallel with Phase 3.
- **Phase 5** (T018–T024): Depends on Phase 2. US3. Can run in parallel with Phases 3–4.
- **Phase 6** (T025–T029): Depends on Phases 3–5 all complete.

### Within Each User Story

- Rule `.scm` file (T007/T012/T018) before registering in `builtin_rules()` (T008/T013/T019) — registration references the include path
- Registration before integration tests can pass
- Positive tests before negative tests (confirm rule fires first, then confirm it doesn't over-fire)

### Parallel Opportunities

- After Phase 2 completes, Phases 3, 4, and 5 can all start simultaneously
- T012 and T013 (Phase 4 file + registration) can run in parallel with T007/T008 (Phase 3)
- T018 and T019 (Phase 5) can run in parallel with Phases 3 and 4
- T025, T026, T027 (JSON contract tests in Phase 6) can run in parallel with each other

---

## Parallel Execution Example: After Phase 2

```text
# All three rules can be written simultaneously:
Task: T007 — Create cartesian_product.scm
Task: T012 — Create deprecated_id_function.scm  [P]
Task: T018 — Create dynamic_property.scm         [P]

# Then register all three:
Task: T008 — Register CartesianProduct
Task: T013 — Register DeprecatedFunction         [P]
Task: T019 — Register DynamicProperty            [P]

# Then write all tests in parallel per story:
Task: T009–T011 (US1 tests)
Task: T014–T017 (US2 tests)  [P]
Task: T020–T024 (US3 tests)  [P]
```

---

## Implementation Strategy

### MVP First (Phase 2 + Phase 3 = CartesianProduct only)

1. Complete Phase 1: Baseline check
2. Complete Phase 2: `Rule.code` + `Code:` header support
3. Complete Phase 3: `CartesianProduct` rule + tests
4. **STOP and VALIDATE**: `cargo test` + `cypher lint -e "MATCH (a), (b) RETURN a, b"` produces warning with `code: "03N90"`
5. Ship or continue to Phases 4–5

### Incremental Delivery

1. Phase 1 + 2 → `code` field infrastructure in place
2. Phase 3 → `CartesianProduct` ships (most impactful rule)
3. Phase 4 → `DeprecatedFunction` ships
4. Phase 5 → `DynamicProperty` ships
5. Phase 6 → polish, JSON contract verification

---

## Notes

- [P] tasks touch different files and have no incomplete-task dependencies — safe to run concurrently
- All three rule `.scm` files follow the format in `contracts/rule-file-format.md`
- `DeprecatedRelationshipTypeList` is intentionally absent — already caught as `ParseError` (see `research.md`)
- The `Diagnostic.code` field is `skip_serializing_if = "Option::is_none"` in `gram-diagnostics` — existing rules without a `Code:` header will continue to emit no `code` field in JSON (backward compatible)
- Total tasks: 29 across 6 phases
