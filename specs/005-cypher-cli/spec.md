# Feature Specification: Cypher CLI

**Feature Branch**: `005-cypher-cli`  
**Created**: 2026-05-08  
**Status**: Draft  
**Input**: User description: "a cypher CLI similar in structure to ../tree-sitter-gram/tools/gram/ with sub-commands for working with .cypher files. Core, bundled sub-commands will include linting as described in proposals/cypher-linter.md"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Lint a Cypher File (Priority: P1)

A developer working with Cypher queries wants to catch common mistakes before running them against a database. They run `cypher lint` on one or more `.cypher` files and receive actionable diagnostics with file names, line numbers, and suggestions.

**Why this priority**: Linting is the core value proposition of the CLI and delivers immediate, tangible value on its own. All other sub-commands extend from this foundation.

**Independent Test**: Run `cypher lint path/to/query.cypher` against a file with known issues (unlabelled node, unbounded relationship, undeclared parameter). Verify that the output identifies each issue with its location and a fix suggestion, and that the exit code is non-zero when errors are found.

**Acceptance Scenarios**:

1. **Given** a `.cypher` file with an unlabelled node pattern `(n)`, **When** `cypher lint file.cypher` is run, **Then** a warning is printed with the line/column of `(n)` and a message suggesting a label.
2. **Given** a `.cypher` file with a variable-length relationship `[*]`, **When** `cypher lint file.cypher` is run, **Then** an error is printed with the location and a message to add a depth limit.
3. **Given** a `.cypher` file with a `/** */` doc comment declaring `@param name` but `$name` not used in the query body, **When** `cypher lint file.cypher` is run, **Then** a warning is printed for the unused parameter.
4. **Given** a `.cypher` file with `$limit` used in the query but no `@param limit` in the doc comment, **When** `cypher lint file.cypher` is run, **Then** a warning is printed for the undocumented parameter.
5. **Given** a `.cypher` file with no issues, **When** `cypher lint file.cypher` is run, **Then** no diagnostics are printed and the exit code is zero.
6. **Given** `--json` flag is passed, **When** `cypher lint --json file.cypher` is run, **Then** output is a machine-readable JSON document with the same diagnostics.
7. **Given** `--strict` flag is passed, **When** warnings are present, **Then** the exit code is non-zero.

---

### User Story 2 - Lint Multiple Files and Directories (Priority: P2)

A developer wants to lint all `.cypher` files in their project in one command, for use in CI pipelines and pre-commit hooks.

**Why this priority**: Batch processing multiplies the value of the lint command and is the natural next step after single-file linting.

**Independent Test**: Run `cypher lint src/queries/` against a directory containing several `.cypher` files. Verify that every file is checked, diagnostics are attributed to the correct files, and the process exits non-zero if any file has errors.

**Acceptance Scenarios**:

1. **Given** a directory of `.cypher` files, **When** `cypher lint <dir>` is run, **Then** all `.cypher` files are checked and results are reported per file.
2. **Given** stdin input via `cat query.cypher | cypher lint -`, **When** no path argument is given, **Then** stdin is read and diagnosed.
3. **Given** mixed paths (files and directories), **When** passed to `cypher lint`, **Then** all files are processed without duplication.
4. **Given** a directory with no `.cypher` files, **When** `cypher lint <dir>` is run, **Then** a note is printed and the exit code is zero.

---

### User Story 3 - Parse Tree Inspection (Priority: P3)

A grammar developer wants to inspect the Tree-sitter parse tree of a `.cypher` file to understand how the grammar is interpreting their query — useful during grammar development and rule authoring.

**Why this priority**: Important for contributors and tooling developers but not required for the primary linting use case.

**Independent Test**: Run `cypher lint --tree query.cypher` and verify the output is a valid s-expression parse tree that matches the input query structure.

**Acceptance Scenarios**:

1. **Given** `--tree` flag is passed, **When** `cypher lint --tree file.cypher` is run, **Then** the s-expression parse tree is printed to stdout and no diagnostics are emitted.
2. **Given** `--tree` and `--json` are both passed, **When** the command runs, **Then** an error is shown explaining the flags are mutually exclusive.

---

### User Story 4 - External Sub-command Dispatch (Priority: P4)

A developer has installed a third-party `cypher-format` binary. When they run `cypher format file.cypher`, the CLI discovers and executes `cypher-format` from their PATH, passing remaining arguments through transparently.

**Why this priority**: Enables the ecosystem to grow without requiring changes to the core CLI binary.

**Independent Test**: Place a script named `cypher-hello` on PATH that prints "hello world". Run `cypher hello`. Verify "hello world" is printed and the exit code matches the script's exit code.

**Acceptance Scenarios**:

1. **Given** `cypher-<name>` exists on PATH, **When** `cypher <name> [args]` is run, **Then** the external binary is exec'd with the remaining args.
2. **Given** no matching built-in or external binary, **When** `cypher <name>` is run, **Then** a helpful error is shown listing available built-in sub-commands.

---

### Edge Cases

- What happens when a `.cypher` file has a parse error? The linter should report a parse error diagnostic and still attempt to emit any structural diagnostics recoverable from the partial tree.
- What happens when a doc comment is present but empty? No contract warnings should fire — an empty comment is not a contract.
- What happens when `--rule <name>` is given but the rule does not exist? A clear error naming the missing rule, with exit code 2.
- How does the linter handle very large files (e.g., a single file with hundreds of statements)? All statements must be checked; performance should not degrade quadratically.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CLI MUST provide a `lint` sub-command that parses `.cypher` files using the Tree-sitter Cypher grammar and emits diagnostics with file name, line, column, severity, rule name, and human-readable message.
- **FR-002**: The `lint` sub-command MUST implement structural rules: unlabelled node patterns and unbounded variable-length relationships (as specified in `proposals/cypher-linter.md`).
- **FR-003**: The `lint` sub-command MUST implement contract rules against cypherdoc `/** */` doc comments: malformed optional parameters and missing tool names.
- **FR-004**: The `lint` sub-command MUST implement cross-reference rules correlating declared `@param` names in doc comments against `$parameter` usages in the Cypher query body (undocumented parameter, unused parameter).
- **FR-005**: The `lint` sub-command MUST accept one or more file paths, one or more directories (recursively finding `.cypher` files), or read from stdin when no paths are given.
- **FR-006**: The `lint` sub-command MUST support a `--json` flag to emit machine-readable output and a `--strict` flag to treat warnings as errors.
- **FR-007**: The `lint` sub-command MUST support a `--tree` flag to print the s-expression parse tree, mutually exclusive with `--json`.
- **FR-008**: The CLI MUST support an `--rule <name>` filter to run only the named rule(s).
- **FR-009**: The CLI MUST dispatch unknown sub-commands to external `cypher-<name>` binaries found on PATH, passing all remaining arguments through unmodified.
- **FR-010**: The CLI MUST exit with code `0` when no diagnostics are found (or only warnings without `--strict`), `1` when errors (or warnings under `--strict`) are found, and `2` for usage errors or internal failures.
- **FR-011**: Lint rules MUST be loaded from `.scm` files with structured comment headers (`Rule:`, `Severity:`, `Applies-to:`, `Message:`) enabling rule addition without recompilation.

### Key Entities

- **Cypher File**: A `.cypher` source file containing one or more `/** */` doc comment + statement pairs, or bare statements.
- **Lint Rule**: A `.scm` Tree-sitter query file with metadata headers declaring its name, severity, applicable tree, and message template.
- **Diagnostic**: A single finding emitted by a rule, carrying rule name, severity, message, and source location (file, line, column, byte range).
- **Doc Comment / Contract**: A `/** */` comment preceding a statement, parsed by the cypherdoc grammar, declaring tool name, parameters, and return shape.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can lint a single `.cypher` file and receive output in under 500 ms on a modern laptop, including grammar loading time.
- **SC-002**: All five rule categories from `proposals/cypher-linter.md` (unlabelled node, unbounded relationship, malformed optional param, missing tool name, undocumented/unused parameter) produce diagnostics in the test suite.
- **SC-003**: The CLI exits non-zero when at least one error-severity diagnostic is found and zero when none are found — verified by integration tests.
- **SC-004**: A developer can add a new rule by placing a single `.scm` file with appropriate headers in a directory and passing `--rules-dir <path>` to `cypher lint`, with no recompilation required.
- **SC-005**: The `--json` output conforms to a documented schema so that CI tools and editors can consume it reliably.
- **SC-006**: Running `cypher lint` against the `references/neo4j-skills/` example files produces no false-positive errors.

## Assumptions

- The CLI is a single Rust binary distributed as a standalone executable (no runtime dependencies beyond the OS).
- Rule `.scm` files are bundled into the binary at compile time for the built-in rules, with an option to load additional rules from a user-specified directory at runtime.
- The `tree-sitter-cypherdoc` sub-grammar is available as a Rust crate (or path dependency within this repo) at the time of implementation.
- The `gram` CLI (`../tree-sitter-gram/tools/gram/`) serves as the structural template: `clap` for argument parsing with the derive API, `ariadne` for pretty diagnostic output, `walkdir` for directory traversal, and external dispatch via `cypher-<name>` binary lookup on PATH.
- Mobile and web targets are out of scope; the CLI targets Linux, macOS, and Windows (x86-64 and ARM64).
- A `cypher skill` sub-command for agent skill management is explicitly deferred to a future feature; the dispatch mechanism will enable it as an external binary in the interim.
