# Research: openCypher BNF Grammar Coverage

**Branch**: `008-opencypher-bnf-gap` | **Date**: 2026-05-10

## Resolved Questions

### 1. How to disambiguate `shortestPath(` from a generic function call

**Decision**: Add `legacy_shortest_path_pattern` as a named rule that is listed before
`function_call` in the `expression` choice, and register
`[$.legacy_shortest_path_pattern, $.function_call]` in the `conflicts` array for GLR resolution.

**Rationale**: Tree-sitter's GLR parser handles ambiguous alternatives by trying all paths and
using the conflict hint to prefer one. Since `legacy_shortest_path_pattern` requires its argument
to be a path pattern (node → rel → node), not a comma-separated expression list, the GLR resolver
will select the correct alternative once it sees the pattern structure. Declaring the conflict
explicitly makes the ambiguity intentional and prevents a generation error.

**Alternatives considered**:
- Making `shortestpath` / `allshortestpaths` reserved keywords — rejected because they are
  `<non_reserved_word>` in the BNF and must remain usable as identifiers in other positions.
- Using an `inline` rule to merge with `function_call` — rejected because it loses the named
  `legacy_shortest_path_pattern` node that downstream tools need for accurate lint detection.

**BNF reference**: `<legacy shortest path pattern>` at line 345 of `openCypher.bnf`

---

### 2. How to parse map projection without conflicting with `map_literal`

**Decision**: Add `map_projection` as a rule of the form
`prec.left(10, seq($.expression, '{', commaSep($.map_projection_element), '}'))` and register
`[$.map_projection, $.map_literal]` in `conflicts`.

**Rationale**: The key structural difference is that `map_projection` has an `expression` before
the `{`, while `map_literal` is standalone. With `prec.left(10)` (matching property access and
subscript precedence), the GLR resolver prefers `map_projection` when an identifier is followed by
`{`. `map_literal` remains the parse for `{` appearing alone in expression position.

**Alternatives considered**:
- Treating map projection as a postfix operator on `expression` via `subscript_expression` —
  rejected because projection elements (`.prop`, `.*`) have different syntax from subscript indices.
- Inlining projection into `function_call` — rejected; it is not a function call per the BNF.

**BNF reference**: `<map projection>` at the map projection section of `openCypher.bnf`

---

### 3. Inline WHERE inside node/relationship patterns — conflict risk

**Decision**: Add `optional($.where_clause)` as the last optional child in `node_pattern` and in
each branch of `relationship_body`. No new conflict registration is needed because the existing
`[$.node_pattern, $.expression]` conflict already causes the GLR parser to explore both
interpretations when a `(` is followed by an identifier.

**Rationale**: After the parser has consumed the node body (variable, label, properties), a
`WHERE` keyword is unambiguous — it cannot start an expression in this position. The existing
conflict entry is sufficient.

**Risk**: Inline WHERE makes the grammar more permissive (allows WHERE in positions where the BNF
only requires it in `<element_pattern_predicate>`). This is acceptable for a parser grammar; semantic
validation (which positions are legal for inline WHERE) is left to the linter layer.

**BNF reference**: `<element_pattern_where_clause>`, `<element_pattern_predicate>` in `openCypher.bnf`

---

### 4. GQL path-search prefix keywords (`ALL`, `ANY`, `SHORTEST`) conflict with identifiers

**Decision**: Use `kw(...)` (case-insensitive non-reserved tokens) for all prefix keywords. The
prefix is parsed as an optional first child of `match_clause`, placed before `$.pattern`. Since
`kw('ALL')` etc. are not reserved in the identifier rule, the `match_clause` context is enough
for the parser to commit: if `MATCH` is followed by `ALL`, `ANY`, or `SHORTEST` (not followed by
`(` which would start a pattern), it must be a path-search prefix.

**Rationale**: The current grammar already uses `kw(...)` for non-reserved words throughout. The
lookahead from the MATCH context plus the `(` vs keyword distinction is sufficient for LR(1) or GLR
disambiguation. No conflict registration should be needed for the most common cases, but if
`MATCH ANY (` is ambiguous (is `ANY` a prefix or a path variable named `any`?), a
`[$.path_search_prefix, $.pattern]` conflict entry will resolve it via GLR.

**BNF reference**: `<path_search_prefix>` and children in `openCypher.bnf`

---

### 5. Quantified path patterns — parenthesized sub-path vs parenthesized expression

**Decision**: Add `quantified_path_primary` as a rule requiring
`'(' node_pattern repeat1(seq(relationship_pattern, node_pattern)) ')' graph_pattern_quantifier`.
The `repeat1` requirement on relationships ensures the content cannot be mistaken for a
parenthesized expression or a `node_pattern` alone.

**Rationale**: A quantified sub-path must contain at least one relationship, which is the same
disambiguation rule used by `pattern_predicate` and `pattern_comprehension`. This requirement
makes the parse unambiguous: `((n))` is a parenthesized node (not a quantified path) and
`((n)-[r]->(m)){2}` is a quantified path.

**BNF reference**: `<quantified_path_primary>`, `<graph_pattern_quantifier>` in `openCypher.bnf`

---

### 6. INF, INFINITY, NAN — keyword vs identifier

**Decision**: Add `inf_literal`, `infinity_literal`, and `nan_literal` as dedicated rules using
`kw(...)` tokens, and list them in `expression` *before* `$.identifier`. Tree-sitter's longest-match
rule means the `kw(...)` token will win over the identifier regex when the input is exactly `INF`,
`INFINITY`, or `NAN`.

**Rationale**: The BNF lists these as `<non_reserved_word>` that appear in the
`<signed_numeric_literal>` production. Making them dedicated literal rules (rather than
identifier aliases) preserves them as queryable named nodes in the parse tree, which is required
for syntax highlighting and semantic analysis.

**BNF reference**: `<signed_numeric_literal>` in `openCypher.bnf`

---

### 7. Existing TCK coverage for new features

**Finding**: The current TCK snapshot at `references/openCypher/tck/features/` does not include
any `shortestPath` or `allShortestPaths` examples (grep returned 0 matches). GQL path-search
prefix and quantified path pattern TCK tests are also absent. Coverage for these features will
come from hand-written corpus tests.

Map projection examples are not present in the TCK map expression features either. The TCK
`expressions/map/` directory covers map literals and property access, not projection.

**Implication**: The TCK gate for Slices 1, 3a, 3b, and 2b will be satisfied by manual corpus
tests rather than TCK-sourced queries. This is acceptable per the constitution: "TCK validation
is the final acceptance gate for each implementation slice" — if no TCK queries exist for the
feature, the gate is satisfied by the absence of failures rather than positive validation.

---

### 8. `||` string concatenation operator

**Finding**: The `||` operator is already handled in `binary_expression` at precedence level 6
(alongside `+` and `-`). It parses correctly but is not a distinctly named operator node in the
tree. This is an existing partial-coverage issue documented in the gap analysis but is out of scope
for this feature — no change planned.
