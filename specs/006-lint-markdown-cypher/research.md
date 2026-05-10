# Research: Lint Markdown Cypher Snippets

**Feature**: 006-lint-markdown-cypher | **Date**: 2026-05-10

## Decision 1 — Extraction Method: Regex vs tree-sitter-markdown vs Line State Machine

**Decision**: Use a line-based state machine (no external dependency).

**Rationale**: The use case is precisely scoped — find ` ```cypher ` fences in markdown files,
extract their content, and record the starting line number. A line-based state machine handles
all realistic cases:

- Standard fences: ` ```cypher `
- Mixed-case: ` ```Cypher `, ` ```CYPHER `
- Fences with trailing annotation: ` ```cypher title="example" `
- Empty blocks
- Unclosed fences (graceful degradation)
- Multiple fences per file

The `regex` crate would also work but adds a compile-time dependency for a feature that is
trivially expressible as a `for line in source.lines()` loop with a boolean state variable.
The line-state machine is easier to read, easier to test, and consistent with the existing
codebase style (no regex usage elsewhere in `tools/cypher/`).

`tree-sitter-markdown` (the split-parser variant) would be authoritative for all CommonMark
edge cases (escaped backticks, fences inside HTML blocks, indented code blocks) but:
1. Adds ~2 MB compiled weight and a C/Rust compilation dependency
2. Requires maintaining a WASM build if the playground is extended to support markdown preview
3. The edge cases it handles (escaped fences in raw HTML) never appear in documentation that
   embeds Cypher queries in practice

**Alternatives considered**:
- `regex` crate — functional but an unnecessary dependency for a loop-based extraction
- `tree-sitter-markdown` — correct but disproportionate for the goal; deferred to a future
  feature if edge case reports emerge

---

## Decision 2 — Diagnostic Line Number Adjustment Strategy

**Decision**: Run `analyze()` on the snippet text, then offset every diagnostic's line numbers
by `snippet.start_line` before storing in the merged `SourceResult`.

**Rationale**: The existing `analyze()` function produces line numbers relative to whatever
source text it is given. If given a snippet `"MATCH (n) RETURN n"` starting at markdown line 42,
`analyze()` produces `range.start.line = 0`. Adding `42` gives the correct absolute line.

The `SourceResult.source` field is set to the **full markdown text** so that `print_pretty()`
(which calls `line_col_to_byte(r.source, line, col)` for ariadne rendering) correctly maps the
adjusted absolute line numbers back to bytes in the markdown file. This gives ariadne the context
to render the surrounding markdown lines, not just the snippet text.

All snippet diagnostics for a single markdown file are merged into one `SourceResult` before
output, so ariadne renders the file once rather than once per snippet.

**Alternatives considered**:
- Store `source = snippet_text` and use raw (unadjusted) line numbers — ariadne renders only
  the snippet, not the surrounding markdown context. Confusing for users who see line 1 for
  every snippet.
- Add a `line_offset` field to `SourceResult` and adjust at render time — more invasive change
  to the existing type; the post-analyze adjustment is simpler and keeps `SourceResult` clean.

---

## Decision 3 — Language Tag Matching

**Decision**: Match the fence language tag case-insensitively against the exact word `cypher`.
A tag of ` ```cypher-shell ` or ` ```cypherdoc ` does not match.

**Rationale**: Case-insensitive matching handles authoring variation without being broad.
Requiring an exact match (after lowercasing) avoids false positives from `cypher-shell` blocks
which have different syntax. Splitting on whitespace and taking the first token handles
` ```cypher title="example" ` style annotations.

---

## Decision 4 — Stdin Behavior

**Decision**: Stdin input is unchanged — it operates on Cypher text directly and does not
attempt markdown extraction.

**Rationale**: There is no reliable way to know whether stdin contains markdown or Cypher without
a content-type hint. Adding `--stdin-format markdown` is a valid future extension but out of
scope here. The existing behavior (treat stdin as Cypher) is preserved.

---

## Decision 5 — `--no-markdown` Default

**Decision**: By default markdown files are included in directory scans. `--no-markdown` is
opt-out.

**Rationale**: The feature is most useful in its default-on state; users who want to skip
markdown can pass the flag. This is consistent with how `cypher lint .` naturally lints
everything in scope.
