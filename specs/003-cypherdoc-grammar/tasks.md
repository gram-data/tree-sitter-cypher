# Tasks: Cypherdoc Injection Grammar

**Input**: Design documents from `specs/003-cypherdoc-grammar/`
**Branch**: `003-cypherdoc-grammar`

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (US1–US4)

---

## Phase 1: Setup (Scaffold)

**Purpose**: Create the `tree-sitter-cypherdoc/` subdirectory and verify the toolchain works
before any grammar rules are written.

- [x] T001 Create directory structure: `tree-sitter-cypherdoc/queries/` and `tree-sitter-cypherdoc/test/corpus/`
- [x] T002 [P] Write `tree-sitter-cypherdoc/package.json` with name `"tree-sitter-cypherdoc"`, type `"module"`, and dev dependency on `tree-sitter-cli ^0.26.5`
- [x] T003 [P] Write `tree-sitter-cypherdoc/tree-sitter.json` registering language name `"cypherdoc"` with `"injection-regex": "cypherdoc"`
- [x] T004 Write `tree-sitter-cypherdoc/grammar.js` with a minimal `document` rule matching `/**` ... `*/` delimiters and no internal content
- [x] T005 Run `tree-sitter generate` from `tree-sitter-cypherdoc/` and add a trivial positive corpus test (`/** */` → `(document)`) plus a negative test (`/* */` → ERROR) to `tree-sitter-cypherdoc/test/corpus/names.txt`; confirm `tree-sitter test` passes

**Checkpoint**: `tree-sitter generate` and `tree-sitter test` succeed with the stub grammar.

---

## Phase 2: Foundational (Name and Description)

**Purpose**: Core infrastructure that US1 and US2 both depend on — the extras pattern for
stripping `*` decorations, and the `name`/`description` nodes that head every document.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T006 Add `extras` to `tree-sitter-cypherdoc/grammar.js`: `[ /\s+/, /[ \t]*\*[ \t\n]/ ]` to strip decorative line prefixes without consuming `*/`
- [x] T007 Implement `name` rule in `tree-sitter-cypherdoc/grammar.js`: first non-empty, non-tag content line matching `[a-zA-Z_][a-zA-Z0-9_]*`
- [x] T008 Implement `description` and `description_line` rules in `tree-sitter-cypherdoc/grammar.js`: prose lines not starting with `@`, ` `, or `*`, accumulated before the first tag
- [x] T009 Add corpus tests to `tree-sitter-cypherdoc/test/corpus/names.txt`: name-only, name + description, multi-paragraph description, and the case starting directly with a tag
- [x] T010 Run `tree-sitter generate` + `tree-sitter test` from `tree-sitter-cypherdoc/` and confirm all corpus tests pass

**Checkpoint**: `/** find_person_by_name\n * Description text\n */` parses to a `document` with `name` and `description` children.

---

## Phase 3: User Story 1 — Parse Named, Documented Cypher Tool (Priority: P1) 🎯 MVP

**Goal**: A `@param` entry with a scalar type (including optional type argument) and either a
required or optional-with-default parameter name parses correctly and is addressable via
Tree-sitter query patterns.

**Independent Test**: `tree-sitter parse cypher/find_person.cypher` and
`tree-sitter parse cypher/get_colleagues.cypher` produce `doc_comment` subtrees with
`param_tag`, `type_annotation`, `scalar_type`, and `required_param`/`optional_param` nodes
and zero ERROR nodes.

- [x] T011 [US1] Implement `type_annotation`, `scalar_type`, and `identifier` rules in `tree-sitter-cypherdoc/grammar.js` — scalar type only, no type argument yet
- [x] T012 [US1] Implement `required_param` and `param_tag` rules in `tree-sitter-cypherdoc/grammar.js`; wire `param_tag` into the `document` rule
- [x] T013 [US1] Implement `optional_param` and `param_default` rules in `tree-sitter-cypherdoc/grammar.js` covering `string_default` (`"..."` / `'...'`), `number_default` (integer and decimal, optionally negative), and `boolean_default` (`true`/`false`)
- [x] T014 [US1] Extend `scalar_type` with `type_argument` rule in `tree-sitter-cypherdoc/grammar.js` — `<identifier>` suffix covering `node<Label>`, `relationship<TYPE>`, `list<scalar_type>`
- [x] T015 [P] [US1] Add corpus tests for `@param` with all plain scalar types (`string`, `integer`, `float`, `boolean`, `path`, `map`, `any`) and a negative test for `@param` with no type to `tree-sitter-cypherdoc/test/corpus/tags.txt`
- [x] T016 [P] [US1] Add corpus tests for `optional_param` with each default type (string, integer, boolean, negative number) and a negative test for bare `[name]` without `=default` to `tree-sitter-cypherdoc/test/corpus/tags.txt`
- [x] T017 [P] [US1] Add corpus tests for `type_argument` covering `node<Person>`, `relationship<KNOWS>`, `list<string>`, and a negative test for `node<>` (empty argument) to `tree-sitter-cypherdoc/test/corpus/types.txt`
- [x] T018 [US1] Run `tree-sitter generate` + `tree-sitter test` from `tree-sitter-cypherdoc/`; run `tree-sitter parse cypher/find_person.cypher` and `cypher/get_colleagues.cypher` from repo root; confirm zero ERROR nodes in `doc_comment` subtrees

**Checkpoint**: US1 fully functional. `find_person.cypher` and `get_colleagues.cypher` parse cleanly with complete `param_tag` structure.

---

## Phase 4: User Story 2 — Named Tuple Return Shape (Priority: P1)

**Goal**: A `@returns` entry with a named tuple type — including multi-member tuples and the
`[]` array marker for many-row cardinality — parses correctly and is addressable via query
patterns.

**Independent Test**: `tree-sitter parse cypher/shortest_path.cypher` and
`tree-sitter parse cypher/get_colleagues.cypher` produce `returns_tag` nodes containing
`tuple_type` with correct `tuple_member` children and an `array_marker` where present.

- [x] T019 [US2] Implement `tuple_member` rule in `tree-sitter-cypherdoc/grammar.js`: `column: (identifier) ':' type: (scalar_type)` with named fields
- [x] T020 [US2] Implement `tuple_type` rule in `tree-sitter-cypherdoc/grammar.js`: comma-separated `tuple_member` list wrapped in `[...]`, with optional `array_marker` child; define `array_marker` as the atomic two-character token `"[]"`
- [x] T021 [US2] Implement `returns_tag` rule in `tree-sitter-cypherdoc/grammar.js`: `@returns` followed by `type_annotation` (tuple_type only) and optional `tag_description`; wire into `document` rule after all `param_tag` nodes
- [x] T022 [P] [US2] Add corpus tests for `@returns` with single-member tuple (no `array_marker`) and `@returns` with multi-member tuple plus `array_marker` to `tree-sitter-cypherdoc/test/corpus/tags.txt`
- [x] T023 [P] [US2] Add negative corpus tests for `@returns` with a bare scalar type (not a tuple) and `@returns` with an empty tuple `[]` to `tree-sitter-cypherdoc/test/corpus/tags.txt`
- [x] T024 [US2] Run `tree-sitter generate` + `tree-sitter test` from `tree-sitter-cypherdoc/`; run `tree-sitter parse` on `cypher/shortest_path.cypher`, `cypher/get_colleagues.cypher`, and `cypher/find_person.cypher`; confirm zero ERROR nodes

**Checkpoint**: US2 fully functional. All five `cypher/*.cypher` example files parse cleanly end-to-end.

---

## Phase 5: User Story 3 — Injection Wiring (Priority: P2)

**Goal**: Editors and tools that support Tree-sitter language injection automatically parse
`/** */` blocks in `.cypher` files using the cypherdoc grammar with no manual configuration.

**Independent Test**: `queries/injections.scm` in `tree-sitter-cypher` matches `doc_comment`
nodes and sets `injection.language "cypherdoc"`; all 102 existing Cypher corpus tests pass
without regression.

- [x] T025 [US3] Verify `queries/injections.scm` in `tree-sitter-cypher` contains the injection pattern for `doc_comment` → `"cypherdoc"` (already written; this is a confirmation task)
- [x] T026 [US3] Verify `tree-sitter-cypherdoc/tree-sitter.json` `"injection-regex"` matches the string `"cypherdoc"` used in `injections.scm`
- [x] T027 [US3] Run `tree-sitter test` from the `tree-sitter-cypher` repo root; confirm all 102 Cypher corpus tests still pass with no regression from the injection wiring

**Checkpoint**: US3 complete. Injection is verified; no regressions in the Cypher grammar.

---

## Phase 6: User Story 4 — Syntax Highlighting (Priority: P3)

**Goal**: An editor using the cypherdoc `highlights.scm` applies distinct highlight groups to
tool name, tag keywords, type annotations, identifiers, defaults, and descriptions.

**Independent Test**: `tree-sitter highlight` on any of the five `cypher/*.cypher` example
files shows the tool name, `@param`/`@returns` keywords, type names, param names, default
values, and description text each captured under distinct capture names.

- [x] T028 [P] [US4] Write `tree-sitter-cypherdoc/queries/highlights.scm` with captures: `(name) @name`, `"@param" @tag`, `"@returns" @tag`, `(scalar_type (identifier) @type)`, `(required_param (identifier) @variable)`, `(optional_param (identifier) @variable)`, `(param_default) @constant`, `(tag_description) @comment`, `(description) @comment`
- [x] T029 [P] [US4] Write `tree-sitter-cypherdoc/queries/tags.scm` with `(document (name) @name)` for symbol indexing
- [x] T030 [US4] Run `tree-sitter highlight` on `cypher/hello.tool.cypher`, `cypher/find_person.cypher`, and `cypher/get_colleagues.cypher` from `tree-sitter-cypherdoc/`; confirm each capture class appears and no ERROR nodes exist

**Checkpoint**: US4 complete. All four user stories are independently functional.

---

## Phase 7: Polish & End-to-End Validation

**Purpose**: Complete the dual-coverage gate (negative tests), run the full validation sweep,
and update developer documentation.

- [x] T031 [P] Add negative corpus tests for each remaining edge case to `tree-sitter-cypherdoc/test/corpus/`: tag description with no `-` separator → ERROR node, `@param` with tuple type instead of scalar → ERROR node
- [x] T032 [P] Update `CLAUDE.md` to document `tree-sitter-cypherdoc/` development commands: `cd tree-sitter-cypherdoc && tree-sitter generate`, `tree-sitter test`, `tree-sitter parse ../cypher/<file>.cypher`
- [x] T033 Run full end-to-end sweep: `tree-sitter test` from `tree-sitter-cypherdoc/` (all cypherdoc corpus tests pass) then `tree-sitter test` from repo root (all 102 Cypher corpus tests pass) then `tree-sitter parse` on all five `cypher/*.cypher` files (zero ERROR nodes in all `doc_comment` subtrees)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — blocks Phases 3 and 4
- **Phase 3 (US1)**: Depends on Phase 2
- **Phase 4 (US2)**: Depends on Phase 3 (tuple members use scalar_type from US1)
- **Phase 5 (US3)**: Depends on Phase 4 (injection only meaningful once grammar is complete)
- **Phase 6 (US4)**: Can start after Phase 4 (highlights are independent of injection)
- **Phase 7 (Polish)**: Depends on Phases 5 and 6

### Within Each Phase

- T002 and T003 are parallel (different files)
- T015, T016, T017 are parallel (different test files, no shared state)
- T022 and T023 are parallel (additive tests to same file is fine sequentially, or split to separate files)
- T028 and T029 are parallel (different query files)
- T031 and T032 are parallel (tests vs docs)

---

## Parallel Example: Phase 3 (US1)

```
# After T012 (required_param done), launch in parallel:
Task T013: optional_param + param_default in grammar.js
Task T015: @param corpus tests in test/corpus/tags.txt
Task T017: type_argument corpus tests in test/corpus/types.txt

# After T014 (type_argument done):
Task T016: optional_param corpus tests in test/corpus/tags.txt

# Then sequentially:
Task T018: generate + test + parse validation
```

---

## Implementation Strategy

### MVP (US1 + US2 only — Phases 1–4)

1. Complete Phase 1: Scaffold
2. Complete Phase 2: Name and description
3. Complete Phase 3: US1 — `@param` with types
4. **Validate**: `tree-sitter parse cypher/find_person.cypher` — zero errors
5. Complete Phase 4: US2 — `@returns` named tuple
6. **Validate**: all five `cypher/*.cypher` files — zero errors in doc_comment subtrees
7. **Stop and demo**: the grammar is usable at this point

### Full delivery

Continue with Phase 5 (injection verification), Phase 6 (highlights), Phase 7 (polish).
Each phase adds a complete, independently testable increment.

---

## Notes

- Always run `tree-sitter generate` before `tree-sitter test` after any `grammar.js` change
- Corpus tests must be written to the `tree-sitter-cypherdoc/` test directory, not the parent repo's `test/corpus/`
- `tree-sitter parse` for end-to-end validation is run from the repo root (where the parent grammar is registered) against `cypher/*.cypher` files
- All five example `.cypher` files are the TCK-analog acceptance gate for this grammar
