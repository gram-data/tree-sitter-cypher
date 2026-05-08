# Tasks: Cypher CLI (`cypher-data`)

**Input**: Design documents from `specs/005-cypher-cli/`  
**Branch**: `005-cypher-cli`  
**Tech stack**: Rust stable, clap 4.5 (derive), ariadne 0.6, walkdir 2, serde/serde_json 1, tree-sitter 0.25+, assert_cmd + predicates (integration tests)

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no blocking dependencies)
- **[US#]**: User story this task belongs to (from spec.md)

---

## Phase 1: Setup (Project Scaffold)

**Purpose**: Create the `tools/cypher/` package and wire it into the repo workspace.

- [ ] T001 Verify `tree-sitter-cypherdoc/Cargo.toml` exists and is not already a member of another Cargo workspace, then add `[workspace]` section to root `Cargo.toml` with members `[".","tools/cypher","tree-sitter-cypherdoc"]` and a `[workspace.package]` block mirroring the gram repo layout
- [ ] T002 Create directory tree: `tools/cypher/src/`, `tools/cypher/rules/structural/`, `tools/cypher/rules/contract/`, `tools/cypher/rules/cross_reference/`, `tools/cypher/tests/fixtures/`
- [ ] T003 Create `tools/cypher/Cargo.toml` declaring package `cypher-data`, binary `cypher`, with deps: clap 4.5 (derive+cargo features), ariadne 0.6, walkdir 2, serde 1 (derive), serde_json 1, tree-sitter 0.25, tree-sitter-cypher (path `../..`), tree-sitter-cypherdoc (path `../../tree-sitter-cypherdoc`), directories 5; dev-deps: assert_cmd 2, predicates 3, tempfile 3

**Checkpoint**: `cargo build -p cypher-data` compiles (empty main is fine at this stage)

---

## Phase 2: Foundational (Core Types and Rule Infrastructure)

**Purpose**: Types, rule loading, and CLI skeleton that every user story depends on. Must be complete before any story work begins.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T004 Create `tools/cypher/src/types.rs` with `Severity`, `Position`, `Range`, `Diagnostic`, `FileResult`, `LintResult` — mirror `gram/src/types.rs`; add `rule: String` field to `Diagnostic`; derive `serde::Serialize/Deserialize` on all types
- [ ] T005 [P] Create `tools/cypher/src/dispatch.rs` — external `cypher-<name>` PATH dispatch; copy structure of `gram/src/dispatch.rs` substituting `cypher-` prefix
- [ ] T006 Create `tools/cypher/src/rules.rs` — define `AppliesTo` enum (`Structural | Contract | CrossReference`), `Rule` struct (`name`, `severity`, `applies_to`, `message`, `query: tree_sitter::Query`), and `parse_rule_file(src: &str, language: tree_sitter::Language) -> Result<Rule, String>` that reads `;;`-prefixed header lines then compiles the remainder as a `tree_sitter::Query`
- [ ] T007 Create `tools/cypher/src/main.rs` with `Cli` struct, `Commands` enum (`Lint(lint::LintArgs)`, `#[command(external_subcommand)] External(Vec<String>)`), and `main()` dispatching to `lint::run` or `dispatch::run`; add `mod` declarations for all modules
- [ ] T008 Create stub `tools/cypher/src/lint.rs` with `LintArgs` (all flags from `contracts/cli-interface.md`: `-e/--expression`, `--json`, `--tree`, `--strict`, `--rule`, `--rules-dir`, `paths`) and a `pub fn run(args: LintArgs) -> i32` that returns `0` (will be filled in Phase 3)

**Checkpoint**: `cargo build -p cypher-data` produces a `cypher` binary; `cypher --help` lists the `lint` sub-command and `cypher lint --help` shows all flags

---

## Phase 3: User Story 1 — Single-File Structural Lint (Priority: P1) 🎯 MVP

**Goal**: `cypher lint file.cypher` reports all five rule categories (structural, contract, cross-reference) with correct location, severity, and message; `--json` and `--strict` flags work correctly.

**Independent Test**: Run `cypher lint` on each fixture file in `tools/cypher/tests/fixtures/` and verify exit code and diagnostic output. All five rule types must fire on their respective fixtures; `clean.cypher` must exit 0.

### Structural Rule Files

- [ ] T009 [P] Create `tools/cypher/rules/structural/unlabelled_node.scm` per `proposals/cypher-linter.md` (matches `(node_pattern variable: (identifier) !label)`)
- [ ] T010 [P] Create `tools/cypher/rules/structural/unbounded_relationship.scm` per `proposals/cypher-linter.md`

### Contract Rule Files

- [ ] T011 [P] Create `tools/cypher/rules/contract/optional_param_missing_default.scm` matching `(param_tag (ERROR) @malformed_param)` per `proposals/cypher-linter.md`

### Cross-Reference Rule Files

- [ ] T012 [P] Create `tools/cypher/rules/cross_reference/undocumented_parameter.scm` matching `(parameter) @used_param`
- [ ] T013 [P] Create `tools/cypher/rules/cross_reference/unused_parameter.scm` matching `(param_tag param: [(required_param)(optional_param)] name: (identifier) @declared_param)`

### Rule Registration

- [ ] T014 Update `tools/cypher/src/rules.rs` to export a `builtin_rules() -> Vec<Rule>` function that loads all five `.scm` files via `include_str!()` using the Cypher language for structural/cross-ref rules and the cypherdoc language for contract rules

### Lint Engine — Single File

- [ ] T015 Implement core of `tools/cypher/src/lint.rs`:
  - `parse_source(src: &str) -> tree_sitter::Tree` using `tree-sitter-cypher`
  - `collect_pairs(tree: &Tree, src: &str) -> Vec<DocStatementPair>` — walk `source_file` children pairing adjacent `doc_comment` + `statement` nodes
  - `parse_doc(doc_src: &str) -> tree_sitter::Tree` using `tree-sitter-cypherdoc`
  - `run_structural_rules(rules, tree, src) -> Vec<Diagnostic>`
  - `run_contract_rules(rules, doc_tree, doc_src, doc_start_byte) -> Vec<Diagnostic>` including Rust-layer `MissingToolName` check (test `node.child_by_field_name("name")` byte range)
  - `run_cross_reference_rules(rules, cypher_tree, doc_tree) -> Vec<Diagnostic>` using HashSet set-difference for declared vs used params

### Output Formatting

- [ ] T016 Implement pretty output path in `lint.rs`: use `ariadne` `Report::build` for each `Diagnostic`; match severity to `ReportKind` (error/warning/advice); `byte_to_char` helper; include rule name in report code
- [ ] T017 Implement `--json` output path in `lint.rs`: serialize `LintResult` via `serde_json::to_string_pretty`; conflicts with `--tree` (return exit code 2 if both set)
- [ ] T018 Wire `--strict` flag: treat any warning-severity diagnostic as an error for exit-code purposes

### Fixture Files and Integration Tests

- [ ] T019 [P] Create fixture files in `tools/cypher/tests/fixtures/`: `clean.cypher` (valid query), `unlabelled_node.cypher`, `unbounded_relationship.cypher`, `unused_param.cypher` (cypherdoc `@param` declared but unused), `undocumented_param.cypher` (`$param` used but not declared), `optional_param_error.cypher` (cypherdoc `[name]` bare optional param triggering ERROR node), `missing_tool_name.cypher` (cypherdoc comment with no tool name on first line), `parse_error.cypher` (malformed Cypher with a deliberate syntax error), `empty_doc.cypher` (query preceded by an empty `/** */` comment with no declarations)
- [ ] T020 [P] [US1] Create `tools/cypher/tests/lint_integration.rs` with integration tests using `assert_cmd::Command` for: clean file exits 0, unlabelled node exits 1 with "UnlabelledNode" in output, unbounded relationship exits 1, `--json` produces valid JSON with correct `schema_version`, `--strict` exits 1 when only warnings present, unused param exits 1, undocumented param exits 1, optional param error exits 1 with "OptionalParamMissingDefault" in output, missing tool name exits 1 with "MissingToolName" in output, parse error produces a diagnostic and exits 1 (not a panic or silent failure), empty doc comment produces no contract warnings and exits 0

**Checkpoint**: `cargo test -p cypher-data` passes all integration tests; `cypher lint` on every fixture file produces the expected exit code and diagnostic output

---

## Phase 4: User Story 2 — Batch / Directory Lint (Priority: P2)

**Goal**: `cypher lint <dir>` finds and checks all `.cypher` files recursively; `cypher lint` with no args reads stdin; mixed path arguments work correctly.

**Independent Test**: Run `cypher lint tools/cypher/tests/fixtures/` and verify all fixture files are checked; exit code reflects the aggregate result.

### Implementation

- [ ] T021 [US2] Extend `lint.rs` `run()` to handle directory paths using `walkdir::WalkDir` filtering on `*.cypher` extension; print a `note: no .cypher files found in <dir>` to stderr and exit 0 when directory is empty of `.cypher` files
- [ ] T022 [US2] Implement stdin mode in `lint.rs`: when `paths` is empty and `-e` is not given, read `io::stdin()` to string; label diagnostics with path `-`
- [ ] T023 [P] [US2] Add integration tests to `lint_integration.rs`: directory mode checks all fixtures and exits non-zero, empty directory exits 0 with note, stdin mode produces diagnostics on a piped unlabelled-node query

**Checkpoint**: `cypher lint tools/cypher/tests/fixtures/` exits 1 and reports diagnostics from every non-clean fixture; `echo 'MATCH (n) RETURN n' | cypher lint` exits 1

---

## Phase 5: User Story 3 — Parse Tree Inspection (Priority: P3)

**Goal**: `cypher lint --tree file.cypher` prints the s-expression parse tree to stdout; `--tree` and `--json` together exit 2 with a clear error.

**Independent Test**: Run `cypher lint --tree tools/cypher/tests/fixtures/clean.cypher` and verify the output is a valid s-expression whose root node is `source_file`.

### Implementation

- [ ] T024 [US3] Implement `--tree` output path in `lint.rs` (depends on T017): parse source, call `tree.root_node().to_sexp()`, print to stdout; the `--json`/`--tree` mutual-exclusion check lives in T017; accept at most one path (exit 2 if more than one path + `--tree`)
- [ ] T025 [P] [US3] Add integration tests to `lint_integration.rs`: `--tree` on a single file outputs s-expression starting with `(source_file`, `--tree --json` together exits 2 with an error message

**Checkpoint**: `cypher lint --tree tests/fixtures/clean.cypher` outputs a valid s-expression

---

## Phase 6: User Story 4 — External Sub-command Dispatch (Priority: P4)

**Goal**: `cypher <name> [args]` exec's `cypher-<name>` from PATH; unknown sub-commands show a helpful error.

**Independent Test**: Create a `cypher-hello` script on PATH that prints "hello world". Run `cypher hello`. Verify "hello world" appears and exit code is 0.

### Implementation

- [ ] T026 [US4] Finalize `tools/cypher/src/dispatch.rs` — `run(args: &[String]) -> i32` with PATH binary lookup and `exec` on Unix / `Command::status` on Windows; error message lists built-in commands when binary not found; mirrors `gram/src/dispatch.rs` exactly
- [ ] T027 [P] [US4] Add integration test to `lint_integration.rs`: unknown sub-command exits 2 with "unknown sub-command" in stderr; (external dispatch with a real stub binary can be a manual test or skipped in CI if PATH manipulation is brittle)

**Checkpoint**: `cypher bogus` exits 2 with the correct error message; external dispatch works on macOS/Linux

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Rule filtering flag, distribution configuration, and documentation.

- [ ] T028 Implement `--rule <NAME>` filtering in `lint.rs`: after loading built-in rules, filter the `Vec<Rule>` to retain only rules whose `name` matches any value in `args.rule`; if a given name matches no loaded rule, print an error to stderr and exit 2
- [ ] T029 [P] Add `--rules-dir <PATH>` support in `lint.rs`: load all `.scm` files from the given directory; append to the built-in rule list; errors loading a rule file print a warning to stderr but do not abort
- [ ] T030 [P] Add `[package.metadata.dist]` targets to `tools/cypher/Cargo.toml` matching gram's distribution targets: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`
- [ ] T031 [P] Create `tools/cypher/README.md` with install, usage examples (one per rule category), and JSON output sample from `contracts/json-output-schema.md`
- [ ] T032 Run `cargo test -p cypher-data` end-to-end to confirm all phases pass; run `cypher lint` against `references/neo4j-skills/` and verify no false-positive errors (SC-006); time `cypher lint <single-fixture-file>` with `time` and assert wall-clock is under 500 ms on the CI runner (SC-001)

**Checkpoint**: `cargo build -p cypher-data --release` succeeds; `cypher --version` prints the correct version; full test suite green

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — **BLOCKS all user stories**
- **Phase 3 (US1, P1)**: Depends on Phase 2 — MVP deliverable
- **Phase 4 (US2, P2)**: Depends on Phase 3 (shares `lint.rs`; directory traversal adds to single-file engine)
- **Phase 5 (US3, P3)**: Depends on Phase 3 (adds `--tree` flag to the same lint command)
- **Phase 6 (US4, P4)**: Depends on Phase 2 only (`dispatch.rs` is independent)
- **Phase 7 (Polish)**: Depends on Phases 3–6

### User Story Dependencies

- **US1 (P1)**: Only depends on foundational (Phase 2)
- **US2 (P2)**: Depends on US1 — extends the same lint engine
- **US3 (P3)**: Depends on US1 — adds a flag to the same lint command
- **US4 (P4)**: Depends on foundational only — entirely independent of US1–US3

### Within Each Phase (Parallel Opportunities)

- T009–T013 (rule `.scm` files): fully parallel, different files
- T016–T018 (output formatters): parallel to each other, each sequential after T015
- T019–T020 (fixtures + tests): fixtures parallel; tests depend on fixtures
- T021–T023 (US2): T021 sequential; T022 sequential; T023 parallel with T021+T022

---

## Parallel Examples

```bash
# Phase 1 — all three setup tasks can run in parallel:
Task: "Add workspace section to root Cargo.toml"                (T001)
Task: "Create tools/cypher/ directory tree"                     (T002)
Task: "Create tools/cypher/Cargo.toml"                         (T003)

# Phase 2 — T004, T005, T006 parallel; T007 and T008 after:
Task: "Create src/types.rs"                                    (T004)
Task: "Create src/dispatch.rs"                                 (T005)
Task: "Create src/rules.rs"                                    (T006)

# Phase 3 rule files — all five in parallel:
Task: "Create rules/structural/unlabelled_node.scm"            (T009)
Task: "Create rules/structural/unbounded_relationship.scm"     (T010)
Task: "Create rules/contract/optional_param_missing_default.scm" (T011)
Task: "Create rules/cross_reference/undocumented_parameter.scm" (T012)
Task: "Create rules/cross_reference/unused_parameter.scm"      (T013)

# Phase 6 can start alongside Phase 3 (dispatch is independent):
Task: "Finalize dispatch.rs"                                   (T026)
```

---

## Implementation Strategy

### MVP First (User Story 1 only)

1. Complete Phase 1 (Setup)
2. Complete Phase 2 (Foundational)
3. Complete Phase 3 (US1 — all structural + contract + cross-ref for single file)
4. **STOP and VALIDATE**: `cargo test -p cypher-data` passes; `cypher lint` on all fixtures correct
5. Ship / demo: `cypher lint` works end-to-end on single `.cypher` files

### Incremental Delivery

1. Phase 1 + 2 → foundation
2. Phase 3 (US1) → **MVP: single-file lint with all rule categories**
3. Phase 4 (US2) → batch lint for CI pipelines
4. Phase 5 (US3) → parse tree for grammar developers
5. Phase 6 (US4) + Phase 7 → ecosystem dispatch + polish

### Parallel Team Strategy

With two contributors (matching gram's 2-person pattern):

1. Both complete Phases 1–2 together
2. **Contributor A**: Phase 3 (US1) → Phase 4 (US2) → Phase 5 (US3)
3. **Contributor B**: Phase 6 (US4, starts at Phase 2 complete) → Phase 7 polish

---

## Notes

- `[P]` = different files, no blocking dependency — safe to run concurrently
- `[US#]` maps each task to spec.md user story for traceability
- No TDD was requested; integration tests are included because the plan explicitly lists them as slice deliverables
- The `MissingToolName` contract rule has no `.scm` file (zero-width MISSING node not queryable) — implemented as a Rust-layer check in `run_contract_rules()` per `research.md` Decision 5
- Rule filtering (`--rule`) and `--rules-dir` are Phase 7 because no user story in P1–P4 requires them; they improve extensibility without blocking the core value
