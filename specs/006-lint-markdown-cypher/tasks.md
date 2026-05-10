# Tasks: Lint Markdown Cypher Snippets

**Input**: Design documents from `specs/006-lint-markdown-cypher/`  
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/cli-interface.md ✓

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[US#]**: Which user story this task belongs to

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Register the new module in the existing project.

- [x] T001 Add `pub(crate) mod markdown;` to `tools/cypher/src/lib.rs` (not `main.rs` — `main.rs` uses the `cypher_data` lib crate, so module registration belongs in `lib.rs`)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: `CypherSnippet` type and `extract_cypher_snippets()` function — required by every user story.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T002 Create `tools/cypher/src/markdown.rs` with `CypherSnippet { content: String, start_line: u32 }` struct and `extract_cypher_snippets(source: &str) -> Vec<CypherSnippet>` using a line-based state machine (see data-model.md for the algorithm)
- [x] T003 Add unit tests inside `tools/cypher/src/markdown.rs` covering: single snippet, multiple snippets, no snippets, empty snippet, mixed-case ` ```Cypher `, fence with trailing annotation ` ```cypher title="x" `, skipped language tags (` ```cql `, ` ```cypher-shell `)
- [x] T004 Verify `cargo test --manifest-path tools/cypher/Cargo.toml markdown` passes

**Checkpoint**: `extract_cypher_snippets` is correct and tested — user story phases can now begin.

---

## Phase 3: User Story 1 — Lint Cypher Snippets in a Markdown File (Priority: P1) 🎯 MVP

**Goal**: `cypher lint README.md` lints all Cypher fenced blocks and reports diagnostics with line numbers relative to the markdown file.

**Independent Test**: Run `cypher lint tests/fixtures/markdown_unlabelled.md` from `tools/cypher/` and verify the output identifies the unlabelled-node warning at the correct markdown line number.

### Implementation for User Story 1

- [x] T005 [P] [US1] Create fixture `tools/cypher/tests/fixtures/markdown_clean.md` — a markdown file with one valid ` ```cypher ` block and surrounding prose
- [x] T006 [P] [US1] Create fixture `tools/cypher/tests/fixtures/markdown_unlabelled.md` — a markdown file with a ` ```cypher ` block containing `MATCH (n) RETURN n` (unlabelled node) at a non-line-1 position (e.g., line 10 inside the markdown)
- [x] T007 [P] [US1] Create fixture `tools/cypher/tests/fixtures/markdown_multi_snippet.md` — a markdown file with three ` ```cypher ` blocks: one clean, one with an unlabelled node, one with an unbounded relationship; each block at different line positions
- [x] T008 [US1] Add `lint_markdown_file(path: &Path, rules: &[Rule]) -> anyhow::Result<SourceResult>` to `tools/cypher/src/lint.rs`: reads full markdown source, calls `extract_cypher_snippets()`, calls `analyze()` per snippet, offsets each diagnostic's `range.start.line` and `range.end.line` by `snippet.start_line`, merges all snippet diagnostics into one `SourceResult` with `source = full_markdown_text` and `path = markdown file path string`
- [x] T009 [US1] Extend the explicit-path branch of `run()` in `tools/cypher/src/lint.rs` to detect `.md` extension and route to `lint_markdown_file()` instead of `analyze()`
- [x] T010 [US1] Add integration tests in `tools/cypher/tests/lint_integration.rs`: `markdown_clean.md` → `success()` with empty stderr; `markdown_unlabelled.md` → `failure()` with stderr containing `"UnlabelledNode"` and the correct absolute line number; `markdown_multi_snippet.md` → `failure()` with diagnostics attributed to correct lines for each block
- [x] T011 [US1] Verify `cargo test --manifest-path tools/cypher/Cargo.toml lint_markdown` passes and `cypher lint tests/fixtures/markdown_unlabelled.md` reports the correct line

**Checkpoint**: `cypher lint <file>.md` fully functional — single markdown file linting works end-to-end.

---

## Phase 4: User Story 2 — Lint All Markdown Files in a Directory (Priority: P2)

**Goal**: `cypher lint docs/` discovers and checks `.md` files alongside `.cypher` files, reporting results per file with correct aggregate exit code.

**Independent Test**: Run `cypher lint tests/fixtures/` from `tools/cypher/` and verify both `.cypher` and `.md` fixtures are checked, with per-file diagnostics attributed correctly.

### Implementation for User Story 2

- [x] T012 [P] [US2] Create fixture `tools/cypher/tests/fixtures/markdown_no_fence.md` — a markdown file with no code blocks (used to verify clean exit for `.md` files without Cypher)
- [x] T013 [US2] Extend the directory-walk branch of `run()` in `tools/cypher/src/lint.rs` to include `extension == "md"` alongside `extension == "cypher"` in the `WalkDir` filter; route `.md` entries to `lint_markdown_file()`
- [x] T014 [US2] Update the "no eligible files found" note in `run()` to say `"no .cypher or .md files found"` instead of `"no .cypher files found"`
- [x] T015 [US2] Add integration tests in `tools/cypher/tests/lint_integration.rs`: directory containing both `.cypher` and `.md` files → both types checked; directory with only `.md` files → snippets checked; `markdown_no_fence.md` in a directory → exit 0 with no diagnostics; mixed directory with one `.md` error → aggregate exit code 1
- [x] T016 [US2] Verify `cargo test --manifest-path tools/cypher/Cargo.toml lint_markdown` still passes with the new directory tests

**Checkpoint**: `cypher lint <dir>` checks both `.cypher` and `.md` files uniformly.

---

## Phase 5: User Story 3 — Suppress Markdown Snippet Checking (Priority: P3)

**Goal**: `cypher lint --no-markdown` skips all `.md` files, with a note when an explicit `.md` path is skipped.

**Independent Test**: Run `cypher lint --no-markdown tests/fixtures/` and verify only `.cypher` fixtures are linted (no mention of `.md` files in output except the skip note for explicit paths).

### Implementation for User Story 3

- [x] T017 [US3] Add `#[arg(long = "no-markdown")] pub no_markdown: bool` field to `LintArgs` struct in `tools/cypher/src/lint.rs`
- [x] T018 [US3] In the directory-walk branch of `run()`, skip files with `.md` extension when `args.no_markdown` is true
- [x] T019 [US3] In the explicit-path branch of `run()`, when an explicit `.md` path is given and `args.no_markdown` is true, print `"note: {path}: skipped (--no-markdown)"` to stderr and continue without adding to results; do not change exit code
- [x] T020 [US3] Add integration tests in `tools/cypher/tests/lint_integration.rs`: `--no-markdown` with directory → only `.cypher` files checked; `--no-markdown` with explicit `.md` path → note printed, exit 0; `--no-markdown` with no `.cypher` files in directory → "no .cypher or .md files found" note, exit 0
- [x] T021 [US3] Verify `cargo test --manifest-path tools/cypher/Cargo.toml` passes (full suite)

**Checkpoint**: All three user stories functional. `--no-markdown` correctly suppresses markdown processing.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Edge-case fixtures, unclosed-fence handling, `--json` contract validation.

- [x] T022 [P] Create fixture `tools/cypher/tests/fixtures/markdown_empty_snippet.md` — a markdown file with an empty ` ```cypher\n``` ` block (only whitespace between fences)
- [x] T023 [P] Add unit test in `tools/cypher/src/markdown.rs` for unclosed fence: last snippet includes all remaining content as `content` with correct `start_line`
- [x] T024 [P] Add integration test for `markdown_empty_snippet.md` → exit 0, no diagnostics (empty snippets silently skipped)
- [x] T025 [P] Add integration test for `--json` with a markdown file: verify the JSON output path field is the `.md` filename, `range.start.line` is the absolute markdown line, and output validates against the schema documented in `specs/005-cypher-cli/contracts/json-output-schema.md`
- [x] T026 Run `cargo test --manifest-path tools/cypher/Cargo.toml` to confirm full suite passes

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — blocks all user stories
- **US1 (Phase 3)**: Depends on Phase 2 completion
- **US2 (Phase 4)**: Depends on Phase 2 + Phase 3 (`lint_markdown_file()` must exist before directory routing is added)
- **US3 (Phase 5)**: Depends on Phase 3 + Phase 4 (flag applies to both explicit-path and directory code paths)
- **Polish (Phase 6)**: Depends on all user story phases

### Within Each Phase

- Fixture creation tasks (T005, T006, T007, T012, T022) are `[P]` — write all fixtures at once
- Implementation tasks that touch `lint.rs` must run sequentially (T008 before T009; T013 before T014)
- Integration tests (T010, T015, T020) follow their implementation tasks

### Parallel Opportunities per Phase

**Phase 3 (US1)**:
```
# Parallel: create all fixtures together
T005: markdown_clean.md
T006: markdown_unlabelled.md
T007: markdown_multi_snippet.md

# Then sequential: implement → test
T008 → T009 → T010 → T011
```

**Phase 4 (US2)**:
```
# Parallel: fixture + implementation can start together
T012: markdown_no_fence.md    (independent)
T013: extend WalkDir filter   (depends only on Phase 2+3)

# Then sequential
T014 → T015 → T016
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002–T004)
3. Complete Phase 3: User Story 1 (T005–T011)
4. **STOP and VALIDATE**: `cypher lint README.md` works end-to-end
5. Demo / merge as incremental value

### Incremental Delivery

1. Phase 1 + Phase 2 → extraction module ready
2. Phase 3 (US1) → single-file markdown linting ← **ship here for fastest feedback**
3. Phase 4 (US2) → directory scan includes `.md`
4. Phase 5 (US3) → `--no-markdown` escape hatch
5. Phase 6 (Polish) → edge cases and JSON contract validation

---

## Notes

- All changes are confined to `tools/cypher/` — no grammar changes, no workspace additions
- The `SourceResult.source` field must be set to the **full markdown text** (not the snippet) so ariadne renders context lines correctly — see data-model.md
- `byte_to_line_col` in `lint.rs` is called with absolute byte positions; after offsetting line numbers the ariadne `line_col_to_byte` call will resolve against the full markdown source correctly
- Commit after each checkpoint (end of each phase) to keep a clean history
