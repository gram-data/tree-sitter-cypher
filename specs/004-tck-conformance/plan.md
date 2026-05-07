# Implementation Plan: TCK Conformance — Complete openCypher Grammar Coverage

**Branch**: `004-tck-conformance` | **Date**: 2026-05-07 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/004-tck-conformance/spec.md`

## Summary

Implement 5 missing grammar features (`is_labeled_expression`, `pattern_comprehension`, `exists_expression`, `pattern_predicate`, path length `*..N` fix) and one minor extension (`<-->` relationship) to eliminate ~86 real TCK failures, raising the non-template TCK pass rate from 80.9% (1309/1617) to ≥98%. Also expand corpus tests from 102 to ≥200 using the gap categories in `proposals/expanded-language-coverage.md`.

All additions are anchored to openCypher BNF productions (see `contracts/grammar-rules.md`). The implementation is grammar-only: edit `grammar.js`, run `tree-sitter generate`, verify `tree-sitter test`, check TCK.

## Technical Context

**Language/Version**: JavaScript (Tree-sitter grammar DSL, tree-sitter-cli ^0.26.5)  
**Primary Dependencies**: `tree-sitter-cli ^0.26.5`, `tree-sitter ^0.25.0` (Node binding)  
**Storage**: N/A — grammar/parser only  
**Testing**: `tree-sitter test` (corpus tests in `test/corpus/`), `tree-sitter parse` on TCK extracts  
**Target Platform**: Any Tree-sitter host (Neovim, Helix, Zed, VS Code, CLI tools)  
**Project Type**: Tree-sitter grammar (language parser)  
**Performance Goals**: Parse at Tree-sitter standard throughput (>100 MB/s); no regression on T086 benchmarks  
**Constraints**: Zero regressions on the 102 existing corpus tests; all new rules must have BNF anchors  
**Scale/Scope**: ~6 new/modified grammar rules, ~100 new corpus tests

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate | Status | Notes |
|------|--------|-------|
| **Fidelity gate** | ✅ Ready | All new rules traced to BNF in `contracts/grammar-rules.md` and `research.md` |
| **Dual-coverage gate** | ⏳ Pending | Each new rule requires ≥1 positive + ≥1 negative corpus test |
| **TCK gate** | ⏳ Pending | TCK pass rate must reach ≥98% on non-template queries |

## Project Structure

### Documentation (this feature)

```text
specs/004-tck-conformance/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 research (complete)
├── data-model.md        # Phase 1 design (complete)
├── contracts/
│   └── grammar-rules.md # BNF anchors and test gates (complete)
└── tasks.md             # Phase 2 output (/speckit-tasks command)
```

### Source Code (repository root)

```text
grammar.js               ← source of truth for all changes
test/corpus/
├── literals.txt          (existing — no changes needed)
├── match_return.txt      (existing — no changes needed)
├── patterns.txt          (existing — may add edge cases)
├── expressions.txt       (existing — add is_labeled, pattern_predicate tests)
├── mutations.txt         (existing — no changes needed)
├── pipeline.txt          (existing — no changes needed)
├── merge_call.txt        (existing — no changes needed)
├── union_advanced.txt    (existing — add exists, pattern_comprehension tests)
└── tck_edge_cases.txt    (new — expanded coverage from gap analysis)
queries/
└── highlights.scm        (may need updates for new node types)
```

**Structure Decision**: Extend existing corpus files for features that belong to their domain; create `tck_edge_cases.txt` for the cross-cutting expanded coverage.

## Implementation Slices

### Slice A — Path length `*..N` fix (Priority: P4, smallest blast radius)

**Goal**: `[:T*..2]` parses without ERROR. No new named rule — just fix the `path_length` token regex.

**BNF**: `<path length>` extended to include `*..M` and `*..`.

**Grammar change** (`grammar.js`):
```js
path_length: _ => token(seq(
  '*',
  optional(choice(
    seq(/[0-9]+/, '..', /[0-9]*/),   // *N..M or *N..
    seq('..', /[0-9]*/),             // *..M or *..
    /[0-9]+/,                        // *N (exact)
  )),
)),
```

**Tests** (add to `test/corpus/patterns.txt`):
- Positive: `(a)-[:T*..2]->(b)`, `(a)-[:T*..]->(b)`, `(a)-[:T*1..]->(b)` (already works?)
- Negative: `(a)-[:T*-1]->(b)` (negative lower, already negative test)

**Checkpoint**: `tree-sitter generate` succeeds; `tree-sitter test -f "patterns"` passes.

---

### Slice B — `<-->` bidirectional relationship (Priority: P7, smallest blast radius)

**Goal**: `MATCH (a)<-->(b)` parses without ERROR.

**Grammar change**: Add `<-->` alternative to `relationship_pattern`:
```js
relationship_pattern: $ => choice(
  ...existing...,
  seq('<-', optional(seq('[', optional($.relationship_body), ']')), '->'),
),
```

**Tests** (add to `test/corpus/patterns.txt`):
- Positive: `MATCH (a)<-->(b) RETURN a`, `MATCH (a)<-[r]->(b) RETURN r`
- Negative: `MATCH (a)<->(b)` (missing dashes) → ERROR

**Checkpoint**: `tree-sitter generate` succeeds; `tree-sitter test -f "patterns"` passes.

---

### Slice C — Label predicate expression (Priority: P1, most TCK failures)

**Goal**: `WHERE n:Person`, `WHERE a:A AND b:B`, `RETURN n:Foo AS result` all parse without ERROR.

**Grammar change** (`grammar.js`): Add new rule and add to `expression`:
```js
// In rules: { ... }
is_labeled_expression: $ => prec.left(5, seq(
  $.expression,
  field('label', $.label_expression),
)),

// In conflicts: $ => [...]
[$.is_labeled_expression, $.set_item],
[$.is_labeled_expression, $.remove_item],
```

Add `$.is_labeled_expression` to the `expression` choice list.

**Tests** (add to `test/corpus/expressions.txt`):
- Positive: `WHERE n:Person`, `WHERE n IS Person`, `WHERE n:A&B`, `RETURN n:Foo AS result`, `WHERE n:A AND n.active = true`
- Negative: `WHERE n:` (trailing colon, no label) → ERROR

**Corpus note**: Some existing SET/REMOVE tests may need review to confirm no regression.

**Checkpoint**: `tree-sitter generate` succeeds; `tree-sitter test` passes (all existing + new).

---

### Slice D — Pattern comprehension (Priority: P2)

**Goal**: `[(n)-->() | n.name]` and `[p = (n)-->() | p]` parse without ERROR.

**Grammar change**: Add `pattern_comprehension` rule, add to `expression`:
```js
pattern_comprehension: $ => prec(3, seq(
  '[',
  optional(seq(field('variable', $.identifier), '=')),
  field('pattern', $.path_pattern),
  optional($.where_clause),
  '|',
  field('projection', $.expression),
  ']',
)),
```

**Tests** (add to `test/corpus/union_advanced.txt`):
- Positive: `[(n)-->() | n.name]`, `[p = (n)-->() | p]`, `[(n)-[:T]->(m) WHERE m.active | m.name]`, `size([(n)-->() | 1])` (inside function call)
- Negative: `[(n)-->()]` (no `|` projection) → ERROR

**Checkpoint**: `tree-sitter generate` succeeds; `tree-sitter test` passes.

---

### Slice E — `exists { }` subquery predicate (Priority: P3)

**Goal**: `WHERE exists { (n)-->() }` and `WHERE exists { MATCH (n)-->(m) RETURN true }` parse without ERROR.

**Grammar change**: Add two new rules, add `exists_expression` to `expression`:
```js
exists_expression: $ => seq(
  kw('EXISTS'),
  '{',
  choice($.pattern, $.exists_subquery),
  '}',
),

exists_subquery: $ => repeat1($.statement),
```

Add `$.exists_expression` to the `expression` choice list.

**Tests** (add to `test/corpus/union_advanced.txt`):
- Positive: `WHERE exists { (n)-->() }`, `WHERE exists { (n)-[:T]->() }`, `WHERE exists { MATCH (n)-->(m) RETURN true }`, `WHERE exists { MATCH (n)-->(m) RETURN true } AND n.active = true`
- Negative: `WHERE exists { }` (empty braces) → ERROR, `WHERE exists ( (n)-->() )` (parens) → ERROR

**Checkpoint**: `tree-sitter generate` succeeds; `tree-sitter test` passes.

---

### Slice F — Pattern predicate in boolean context (Priority: P5)

**Goal**: `WHERE (n)-[]->()` and `WHERE NOT (n)-[:T]-()` parse without ERROR.

**Grammar change**: Add `pattern_predicate` rule and conflict, add to `expression`:
```js
pattern_predicate: $ => prec.dynamic(2, seq(
  $.node_pattern,
  choice(
    $.relationship_pattern,
    seq($.relationship_pattern, $.node_pattern, repeat(seq($.relationship_pattern, $.node_pattern))),
  ),
)),
```

Add to `conflicts: $ => [...]`:
```js
[$.pattern_predicate, $.expression],
```

**Tests** (add to `test/corpus/expressions.txt`):
- Positive: `WHERE (n)-[]->()`, `WHERE (n)-[:REL1]-()`, `WHERE (n)-[:REL1*]->()`, `WHERE NOT (n)-->(m)`, `WHERE (n)-[]->(m) AND n.active`
- Negative: pattern_predicate with no relationship following is just a parenthesized expression

**Checkpoint**: `tree-sitter generate` succeeds; `tree-sitter test` passes. Confirm `WHERE (expr)` still works (no regression).

---

### Slice G — Expanded corpus test coverage (Priority: P5, cross-cutting)

**Goal**: Total corpus test count ≥200. Cover gap categories from `proposals/expanded-language-coverage.md`.

**New file**: `test/corpus/tck_edge_cases.txt` with:

**From gap analysis**:
- Chained patterns: `MATCH (a), (b)--(c)`, `MATCH (a)-->(b)-->(c)-->(d)`, `MATCH ()-->(n)`
- Relationship label disjunction: `MATCH (a)-[:KNOWS|LIKES]->(b)`
- Property map in relationship: `MATCH (a)-[r:T {since: 2020}]->(b)`
- Operator precedence: `NOT a AND b`, `a + b * c + d`
- Slice notation: `list[1..3]`, `list[..3]` (if supported by subscript_expression)
- RETURN `*`: `MATCH (n) RETURN *`
- Multiple ORDER BY: `ORDER BY n.name ASC, n.age DESC`
- SET multiple: `SET n.a = 1, n.b = 2`
- DELETE multiple: `DELETE n, m`
- Chained UNION: `A UNION B UNION C`
- Multi-statement separator: `MATCH (n) RETURN n; MATCH (m) RETURN m`

**Additional negative tests** (5+ per existing slice):
- Mid-expression token: `RETURN a + * b` → ERROR
- Keyword as identifier: `MATCH (match) RETURN match` → ERROR (or valid if unquoted)
- Multiple errors in same query

**Checkpoint**: `tree-sitter test` passes 100%, total count ≥200.

---

### Slice H — TCK gate validation

**Goal**: Non-template TCK pass rate ≥98%.

**Steps**:
1. `bash scripts/extract-tck-queries.sh`
2. `tree-sitter parse /tmp/tck-queries/*.cypher 2>/dev/null | grep -c ERROR`
3. Subtract known-intentional-invalid count (~11)
4. Assert result ≤ ~10 remaining (covering only truly unsupported constructs)

**Expected remaining failures** (post-implementation):
- `MATCH (n $param)` — if parameter-in-pattern fix not achieved
- FOREACH queries — deferred
- Quantified path patterns (`[*]+`) — deferred
- Intentionally invalid literals and syntax — these SHOULD fail

**Checkpoint**: Non-template pass rate ≥98%, documented in PR description.

---

## Execution Order

Slices are ordered by risk and blast radius:

```
A (path length fix) → B (bidir rel) → tree-sitter test
C (is_labeled_expression) → tree-sitter test
D (pattern_comprehension) → tree-sitter test
E (exists {}) → tree-sitter test
F (pattern_predicate) → tree-sitter test
G (expanded tests) → tree-sitter test (≥200 total)
H (TCK gate) → assert ≥98% non-template pass rate
```

Slices A and B can be implemented together (both in `relationship_pattern` / `path_length` area). Slices C–F each touch `expression` in `grammar.js` — implement sequentially to avoid conflict pile-up.

## Complexity Tracking

| Deviation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| `prec.dynamic` for `pattern_predicate` | `(n)` is both `node_pattern` and `seq('(', expression, ')')` — static prec cannot resolve | Static prec would require restructuring entire expression hierarchy |
| `exists_subquery` as inline rule | EXISTS body can be either a plain pattern OR a multi-clause pipeline — need a wrapper to name the multi-clause form | Flattening into `exists_expression` would make the tree ambiguous |
