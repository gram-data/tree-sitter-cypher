# Implementation Plan: openCypher BNF Grammar Coverage

**Branch**: `008-opencypher-bnf-gap` | **Date**: 2026-05-10 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/008-opencypher-bnf-gap/spec.md`

## Summary

Extend `grammar.js` to eliminate parse errors on valid openCypher queries by implementing the ten
highest-impact missing grammar rules identified in `proposals/bnf-gap-analysis.md`. The work is
organized into six slices ordered by impact: `shortestPath`/`allShortestPaths` (P1), inline WHERE
in patterns and map projection (P2), GQL path-search prefixes, quantified path patterns, `YIELD …
WHERE`, and numeric literal extensions (P3). Each slice is independently testable and ships with
corpus tests and TCK validation before the next slice begins.

## Technical Context

**Language/Version**: JavaScript (tree-sitter DSL); tree-sitter CLI ^0.25.0; Node.js v24.15.0
**Primary Dependencies**: tree-sitter (grammar generation + test runner), node-gyp (native bindings)
**Storage**: N/A — grammar project; generated parser written to `src/parser.c`
**Testing**: `tree-sitter test` (corpus tests in `test/corpus/`) + `tree-sitter parse` for TCK validation
**Target Platform**: Portable C parser; Node.js and other language bindings
**Project Type**: Parser grammar library
**Performance Goals**: `tree-sitter generate` completes with zero conflicts; `tree-sitter test` runs in < 5 s
**Constraints**: No shift/reduce or reduce/reduce conflicts; no regression on existing corpus tests; TCK queries in scope must parse to zero ERROR nodes
**Scale/Scope**: ~640-line grammar.js; 10 new named rules; 10 corpus test files touched

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate | Status | Notes |
|------|--------|-------|
| **Fidelity gate** | ✅ | All planned rules trace to named BNF productions — verified against `references/openCypher/grammar/openCypher.bnf` |
| **Dual-coverage gate** | ✅ | Plan mandates ≥1 positive + ≥1 negative corpus test per new rule before slice is done |
| **TCK gate** | ✅ | Relevant TCK feature areas mapped per slice; parse validation required before merge |

## Project Structure

### Documentation (this feature)

```text
specs/008-opencypher-bnf-gap/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output — new node types schema
├── contracts/
│   └── node-types.md    # Public parse-tree contract for new nodes
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
grammar.js                      # All grammar changes land here
test/corpus/
├── patterns.txt                # Inline WHERE in node/rel patterns; shortestPath
├── expressions.txt             # Map projection; INF/NAN literals
├── match_return.txt            # GQL path-search prefixes
├── pipeline.txt                # YIELD … WHERE
└── [new] quantified_paths.txt  # Quantified path patterns (new file)
queries/
└── highlights.scm              # New node names require highlight capture rules
```

**Structure Decision**: Single-project grammar. All rule additions go into `grammar.js`; corpus tests
are distributed across existing `.txt` files by topic, with a new `quantified_paths.txt` added for
the quantified path pattern slice.

## Implementation Slices

### Slice 1 — `shortestPath` / `allShortestPaths` (P1)

**BNF anchor**: `<legacy_shortest_path_pattern>`, `<shortest_path_expression>`

Add a `legacy_shortest_path_pattern` rule that matches
`{ SHORTESTPATH | ALLSHORTESTPATHS } '(' node_pattern relationship_pattern node_pattern ')'`
and add it to the `expression` alternatives. Since `shortestPath(` currently lexes as
`identifier '('` (matching `function_call`), a GLR conflict entry is needed:
`[$.legacy_shortest_path_pattern, $.function_call]`. The keyword tokens
`shortestpath` and `allshortestpaths` are distinct enough that the GLR resolver will
always prefer `legacy_shortest_path_pattern` when the argument is a path pattern.

**Files changed**: `grammar.js` (new rule + conflict entry + expression alternative)
**Corpus test file**: `test/corpus/patterns.txt`
**TCK scope**: `references/openCypher/tck/features/useCases/` — any shortestPath queries

---

### Slice 2a — Inline WHERE in node and relationship patterns (P2)

**BNF anchor**: `<element_pattern_where_clause>`, `<element_pattern_predicate>`

Add `optional($.where_clause)` as the last child of `node_pattern` and inside all
`relationship_body` alternatives. The `where_clause` rule already exists; this is an additive
change with no new rules. A conflict entry `[$.node_pattern, $.expression]` already exists and
covers the new WHERE variant.

**Files changed**: `grammar.js` (`node_pattern` and `relationship_body` rules)
**Corpus test file**: `test/corpus/patterns.txt`
**TCK scope**: `references/openCypher/tck/features/clauses/match/`

---

### Slice 2b — Map projection (P2)

**BNF anchor**: `<map_projection>`, `<map_projection_element>`, `<field_selector>`,
`<all_fields_selector>`, `<literal_map_field>`, `<variable_selector>`

Add a `map_projection` rule: `expression '{' commaSep(map_projection_element) '}'`.
`map_projection_element` is a choice of:
- `field_selector`: `'.' identifier` (selects one property)
- `all_fields_selector`: `'.' '*'`
- `literal_map_field`: `identifier ':' expression` (same shape as `property_key_value`)
- `variable_selector`: `identifier` (bare variable)

Add `map_projection` to the `expression` choices. A conflict entry
`[$.map_projection, $.map_literal]` handles the `expression '{` ambiguity, resolved
by GLR because `map_projection` requires an expression before `{` while `map_literal` is standalone.

**Files changed**: `grammar.js` (5 new rules + conflict entry + expression alternative)
**Corpus test file**: `test/corpus/expressions.txt`
**TCK scope**: `references/openCypher/tck/features/expressions/map/`

---

### Slice 3a — GQL path-search prefixes (P3)

**BNF anchor**: `<path_search_prefix>`, `<all_path_search>`, `<any_path_search>`,
`<shortest_path_search>`, `<all_shortest_path_search>`, `<any_shortest_path_search>`,
`<counted_shortest_path_search>`, `<counted_shortest_group_search>`

Add a `path_search_prefix` rule (choice of all the above forms) and make
`match_clause` accept `optional($.path_search_prefix)` before `$.pattern`.
The prefix keywords (`ALL`, `ANY`, `SHORTEST`) are non-reserved, so the grammar uses
`kw(...)` tokens. The GQL prefix applies to the entire MATCH pattern, not per-path.

**Files changed**: `grammar.js` (8 new rules + `match_clause` change)
**Corpus test file**: `test/corpus/match_return.txt`
**TCK scope**: No TCK coverage in the current snapshot (newer spec additions); validate with
manual `.cypher` examples.

---

### Slice 3b — Quantified path patterns (P3)

**BNF anchor**: `<quantified_path_primary>`, `<graph_pattern_quantifier>`,
`<fixed_quantifier>`, `<general_quantifier>`, `<parenthesized_path_pattern_expression>`

Add:
- `graph_pattern_quantifier`: `choice('+', '*', fixed_quantifier, general_quantifier)`
- `fixed_quantifier`: `'{' integer_literal '}'`
- `general_quantifier`: `'{' optional(integer_literal) ',' optional(integer_literal) '}'`
- `quantified_path_primary`: `'(' node_pattern repeat1(seq(relationship_pattern, node_pattern)) ')' graph_pattern_quantifier`

Extend `path_pattern` to allow an optional sequence of `quantified_path_primary` nodes as
part of the path.

**Files changed**: `grammar.js` (4 new rules + `path_pattern` change)
**Corpus test file**: `test/corpus/quantified_paths.txt` (new)
**TCK scope**: No TCK coverage; validate manually.

---

### Slice 3c — `YIELD … WHERE` (P3)

**BNF anchor**: `<yield_clause>` (WHERE trailing form)

Additive one-line change: append `optional($.where_clause)` to `yield_clause`.
No new rules needed.

**Files changed**: `grammar.js` (`yield_clause` rule only)
**Corpus test file**: `test/corpus/pipeline.txt`
**TCK scope**: `references/openCypher/tck/features/clauses/call/`

---

### Slice 4 — Numeric literal extensions (P3)

**BNF anchor**: `<approximate_number_suffix>`, `<unsigned_decimal_integer>` (underscore separators),
`<signed_numeric_literal>` (INF/INFINITY/NAN)

Three small additions:
1. Add `inf_literal`, `infinity_literal`, `nan_literal` keyword rules (case-insensitive tokens)
   and add them to the `expression` choices. They must be added before `identifier` in the
   choice list to avoid being consumed as plain identifiers.
2. Extend `float_literal` regex to allow an optional `[fFdD]` suffix.
3. Extend `integer_literal` and `float_literal` regexes to allow `_` digit separators.

**Files changed**: `grammar.js` (3 new rules + literal regex updates)
**Corpus test file**: `test/corpus/literals.txt`
**TCK scope**: `references/openCypher/tck/features/expressions/literals/`

## Complexity Tracking

*No constitution violations — all additions trace to BNF productions and are additive.*
