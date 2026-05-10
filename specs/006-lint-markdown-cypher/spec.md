# Feature Specification: Lint Markdown Cypher Snippets

**Feature Branch**: `006-lint-markdown-cypher`  
**Created**: 2026-05-10  
**Status**: Draft  
**Input**: User description: "The `cypher lint` should support scanning markdown files for code-fenced cypher snippets and lint those as well. Consider adopting whether a regex is sufficient to extract the snippets, or we need the full tree-sitter-markdown"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Lint Cypher Snippets in a Markdown File (Priority: P1)

A documentation author or developer has a README or tutorial containing Cypher code blocks fenced with ` ```cypher `. They run `cypher lint README.md` and receive diagnostics for any problematic Cypher in those blocks, with locations reported relative to the original markdown file.

**Why this priority**: This is the core user need. Catching broken Cypher in documentation prevents shipping bad examples to users of a library or tutorial.

**Independent Test**: Run `cypher lint README.md` against a markdown file containing one clean Cypher block and one block with an unlabelled node. Verify that only the problematic block produces a diagnostic, the file name is `README.md`, and the line number points into the fenced block (not line 1 of the extracted snippet).

**Acceptance Scenarios**:

1. **Given** a `.md` file with a ` ```cypher ` block containing an unlabelled node, **When** `cypher lint file.md` is run, **Then** a warning is emitted with `file.md`, the correct line number inside the file, and the same rule name as for a `.cypher` file.
2. **Given** a `.md` file with multiple Cypher fenced blocks, **When** `cypher lint file.md` is run, **Then** each block is checked independently and diagnostics from each reference the correct line.
3. **Given** a `.md` file with no Cypher fenced blocks, **When** `cypher lint file.md` is run, **Then** no diagnostics are emitted and the exit code is zero.
4. **Given** `--json` is passed, **When** `cypher lint --json file.md` is run, **Then** the JSON output includes the markdown file path and accurate line/column for each diagnostic.
5. **Given** a `.md` file whose Cypher snippet has a parse error, **When** `cypher lint file.md` is run, **Then** a parse-error diagnostic is emitted pointing to the relevant line in the markdown file.

---

### User Story 2 - Lint All Markdown Files in a Directory (Priority: P2)

A developer runs `cypher lint docs/` and all `.md` files in the directory are scanned alongside any `.cypher` files, with Cypher snippets inside markdown checked by the same rules.

**Why this priority**: Projects keep documentation next to source files; batch mode should treat both file types uniformly.

**Independent Test**: Run `cypher lint docs/` against a directory containing both `.cypher` and `.md` files. Verify that both file types are checked, results are attributed correctly, and the aggregate exit code reflects errors from either type.

**Acceptance Scenarios**:

1. **Given** a directory with `.cypher` and `.md` files, **When** `cypher lint <dir>` is run, **Then** both file types are linted and results are reported per file.
2. **Given** a `.md` file with no Cypher blocks alongside `.cypher` files, **When** `cypher lint <dir>` is run, **Then** the markdown file produces no diagnostics and does not affect the exit code.
3. **Given** only `.md` files (no `.cypher` files) in a directory, **When** `cypher lint <dir>` is run, **Then** snippets from all `.md` files are checked normally.

---

### User Story 3 - Suppress Markdown Snippet Checking (Priority: P3)

A developer wants to lint only `.cypher` files and skip markdown, for performance or because the docs contain intentionally invalid examples.

**Why this priority**: Escape hatch for CI pipelines that manage `.cypher` and `.md` quality separately.

**Independent Test**: Run `cypher lint --no-markdown docs/` with mixed file types. Verify that markdown files are skipped entirely and only `.cypher` files are checked.

**Acceptance Scenarios**:

1. **Given** `--no-markdown` flag, **When** `cypher lint --no-markdown <dir>` is run, **Then** `.md` files are ignored and only `.cypher` files are linted.
2. **Given** `--no-markdown` flag with explicit `.md` path, **When** `cypher lint --no-markdown file.md` is run, **Then** a warning is printed that the file was skipped, exit code is zero.

---

### Edge Cases

- What happens when a fenced block uses an alternate language tag such as ` ```Cypher ` (mixed case) or ` ```cypher-shell `? Tags are matched case-insensitively for `cypher`; `cypher-shell` is treated as a distinct language and skipped.
- What happens when a fenced block is not terminated before end-of-file? The extractor treats the rest of the file as the block content and emits a warning about the unclosed fence.
- What happens when a Cypher snippet is empty (only whitespace)? The snippet is skipped silently — no diagnostics, no errors.
- How are line numbers reported for snippets inside indented list items? Line numbers always refer to the raw line in the source file. Note: blockquote-prefixed fences (lines starting with `> `) are not recognized by the extractor and will be silently skipped — only fences where the opening backticks appear after optional whitespace are matched.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `lint` sub-command MUST accept `.md` files as valid input paths alongside `.cypher` files.
- **FR-002**: The `lint` sub-command MUST extract all code-fenced blocks whose language tag is `cypher` (case-insensitive) from markdown input.
- **FR-003**: Each extracted Cypher snippet MUST be linted using the same rules applied to `.cypher` files.
- **FR-004**: Diagnostics for snippets MUST report the source markdown file path and the line number within that file where the snippet content begins (the line after the opening fence), not a line number within the extracted snippet text.
- **FR-005**: The `lint` sub-command MUST support `--no-markdown` to skip all `.md` files during directory traversal or explicit path processing.
- **FR-006**: When processing directories, `.md` files MUST be discovered alongside `.cypher` files unless `--no-markdown` is set.
- **FR-007**: Lint output format (human-readable and `--json`) MUST be consistent for both `.cypher` and `.md` inputs — a consumer should not need to distinguish the source file type.
- **FR-008**: A snippet with a parse error MUST emit a parse-error diagnostic pointing to the correct line in the markdown file; remaining snippets in the same file MUST still be checked.

### Key Entities

- **Markdown File**: A `.md` source file potentially containing one or more Cypher fenced code blocks.
- **Cypher Snippet**: A fenced code block with language tag `cypher`, extracted from a markdown file with its starting line number recorded for offset mapping.
- **Line Offset Map**: The mapping from snippet-relative line numbers back to markdown-file-absolute line numbers, used to rewrite diagnostic locations.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can run `cypher lint README.md` and receive diagnostics for Cypher snippets with line numbers that match the actual lines in the markdown file — verified by inspection.
- **SC-002**: Linting a markdown file with ten Cypher fenced blocks completes in under 500 ms on a modern laptop, consistent with the single `.cypher` file target.
- **SC-003**: All lint rules that apply to `.cypher` files produce equivalent diagnostics when the same Cypher is embedded in a markdown fence — verified by integration tests comparing `.cypher` and `.md` inputs for each rule.
- **SC-004**: The `--json` output for a markdown file passes the same schema validation as `.cypher` output — no new required fields, no missing fields.
- **SC-005**: Running `cypher lint docs/` on the project's own documentation produces no false positives for intentionally-illustrative Cypher snippets that are valid.

## Assumptions

- The existing `cypher lint` implementation (005-cypher-cli) is complete and provides the linting engine that this feature extends.
- Regex-based extraction is sufficient for extracting fenced code blocks — a full markdown parse tree is not required because the feature only needs to locate ` ```cypher ` fences and their content, not understand the full document structure.
- The language tag comparison is case-insensitive and only matches the exact word `cypher` (not `cypher-shell`, `cypherdoc`, etc.).
- Only files with a `.md` extension are treated as markdown; `.mdx`, `.markdown`, and other variants are not included in the initial implementation.
- The `--json` schema extension (adding a `snippetRange` or similar field) is an optional enhancement; the minimum viable output uses the adjusted absolute line number and the markdown file path.
- Stdin input mode (`cypher lint -`) continues to operate on Cypher text directly and does not attempt markdown extraction.
