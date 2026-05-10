# Feature Specification: openCypher BNF Grammar Coverage

**Feature Branch**: `008-opencypher-bnf-gap`
**Created**: 2026-05-10
**Status**: Draft
**Input**: User description: "Close the gap with the openCypher BNF as described in the @proposals/bnf-gap-analysis.md"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Parse Shortest Path Queries (Priority: P1)

A Cypher developer writes a query using `shortestPath()` or `allShortestPaths()` to find connections in a graph. Today these queries produce parse errors even though they are valid openCypher. After this feature, the grammar parses them correctly so that lint tools, syntax highlighters, and editors handle them without errors.

**Why this priority**: `shortestPath` is the highest-frequency gap — it causes false ParseErrors on valid, commonly written queries today. It is the direct trigger for this work.

**Independent Test**: A file containing `MATCH path = shortestPath((a:Person)-[*]-(b:Person)) RETURN path` can be parsed without any `ERROR` node in the syntax tree.

**Acceptance Scenarios**:

1. **Given** a `.cypher` file using `shortestPath((a)-[*]-(b))`, **When** the file is parsed, **Then** the result contains a `legacy_shortest_path_pattern` node and no `ERROR` nodes.
2. **Given** a `.cypher` file using `allShortestPaths((a)-[:KNOWS*]-(b))`, **When** the file is parsed, **Then** the result contains a `legacy_shortest_path_pattern` node and no `ERROR` nodes.
3. **Given** a `shortestPath` call used as an expression in `MATCH path = shortestPath(...)`, **When** parsed, **Then** the path assignment binds to the `legacy_shortest_path_pattern`, not to a generic function call.

---

### User Story 2 - Write Queries with Inline Pattern Predicates (Priority: P2)

A Cypher developer writing Neo4j 5+ queries uses inline WHERE clauses inside node and relationship patterns (e.g., `(n WHERE n.age > 30)`) to co-locate predicates with their elements. After this feature, the grammar parses these patterns correctly.

**Why this priority**: Inline predicates are part of the openCypher spec and are increasingly common in modern Neo4j queries. Without support, all such queries contain ERROR nodes.

**Independent Test**: A query `MATCH (n WHERE n.active = true) RETURN n` parses with a `where_clause` field inside the `node_pattern` and no ERROR nodes.

**Acceptance Scenarios**:

1. **Given** `MATCH (n WHERE n.age > 30) RETURN n`, **When** parsed, **Then** the `node_pattern` has a `where` field containing a `where_clause` node and no ERROR nodes.
2. **Given** `MATCH ()-[r WHERE r.weight > 5]-() RETURN r`, **When** parsed, **Then** the `relationship_body` has a `where` field containing a `where_clause` node and no ERROR nodes.
3. **Given** a pattern with both inline WHERE and property map `(n:Person {name: $name} WHERE n.active)`, **When** parsed, **Then** both the `properties` field and the `where` field appear as distinct named fields with no ERROR.

---

### User Story 3 - Write Map Projection Expressions (Priority: P2)

A Cypher developer uses map projection syntax (`n { .name, .age }` or `n { .* }`) to shape query output. After this feature, the grammar parses map projections correctly so tooling can analyse and highlight them.

**Why this priority**: Map projection is a heavily used Neo4j pattern for shaping output. Its absence causes parse failures on a large category of real-world queries.

**Independent Test**: A query `RETURN n { .name, .age, score: 10 }` parses with a `map_projection` node and no ERROR nodes.

**Acceptance Scenarios**:

1. **Given** `RETURN n { .name, .age }`, **When** parsed, **Then** the expression contains a `map_projection` node with `field_selector` children and no ERROR nodes.
2. **Given** `RETURN n { .* }`, **When** parsed, **Then** the `map_projection` contains an `all_fields_selector` node.
3. **Given** `RETURN n { .name, score: 10 }`, **When** parsed, **Then** `map_projection` contains both a `field_selector` and a `literal_map_field`.

---

### User Story 4 - Parse GQL-Style Path Search Prefixes (Priority: P3)

A developer writing queries with GQL-aligned path search modes (`MATCH ALL (a)-[*]-(b)`, `MATCH ANY SHORTEST (a)-[*]-(b)`, `MATCH SHORTEST 3 (a)-[*]-(b)`) gets correct parse results rather than errors.

**Why this priority**: These are newer openCypher spec additions (GQL alignment). Important for completeness but less common in production codebases today than P1/P2 features.

**Independent Test**: A query `MATCH ANY SHORTEST (a:Person)-[*]-(b:Person) RETURN a, b` parses with a `path_search_prefix` node and no ERROR nodes.

**Acceptance Scenarios**:

1. **Given** `MATCH ALL (a)-[*]-(b) RETURN a, b`, **When** parsed, **Then** an `all_path_search` node is present with no ERROR.
2. **Given** `MATCH ANY SHORTEST (a)-[*]-(b) RETURN a, b`, **When** parsed, **Then** an `any_shortest_path_search` node is present.
3. **Given** `MATCH SHORTEST 3 (a)-[*]-(b) RETURN a, b`, **When** parsed, **Then** a `counted_shortest_path_search` node with the count `3` is present.

---

### User Story 5 - Parse Quantified Path Patterns (Priority: P3)

A developer uses quantified path patterns (`((a)-[:KNOWS]->(b)){1,3}` or `((a)-->(b))+`) which are part of the openCypher spec. After this feature the grammar parses them without errors.

**Why this priority**: Required for full spec compliance. Increasingly supported in Neo4j 5.x.

**Independent Test**: A query `MATCH ((a)-[:KNOWS]->(b)){2,5} RETURN a, b` parses with a `quantified_path_primary` node and no ERROR nodes.

**Acceptance Scenarios**:

1. **Given** `MATCH ((a)-[r]->(b)){1,3} RETURN a`, **When** parsed, **Then** `quantified_path_primary` with a `general_quantifier` `{1,3}` is present.
2. **Given** `MATCH ((a)-[r]->(b))+ RETURN a`, **When** parsed, **Then** `quantified_path_primary` with a `+` quantifier is present.
3. **Given** `MATCH ((a)-[r]->(b)){3} RETURN a`, **When** parsed, **Then** `quantified_path_primary` with a `fixed_quantifier` `{3}` is present.

---

### User Story 6 - Parse YIELD … WHERE in CALL (Priority: P3)

A developer writes `CALL db.labels() YIELD label WHERE label STARTS WITH 'P'` to filter procedure results inline. After this feature the grammar parses the `WHERE` clause after `YIELD` correctly.

**Why this priority**: Needed for full CALL clause compliance with the spec.

**Independent Test**: `CALL db.labels() YIELD label WHERE label STARTS WITH 'A' RETURN label` parses with a `where_clause` child of `yield_clause` and no ERROR nodes.

**Acceptance Scenarios**:

1. **Given** `CALL db.labels() YIELD label WHERE label STARTS WITH 'A'`, **When** parsed, **Then** the `yield_clause` contains a `where_clause` and no ERROR.
2. **Given** `CALL proc() YIELD *` (no WHERE), **When** parsed, **Then** the `yield_clause` parses correctly without requiring WHERE.

---

### Edge Cases

- A `shortestPath` or `allShortestPaths` call with a variable-length relationship that has a depth limit (e.g., `[*..5]`) must parse correctly.
- Map projection used as a sub-expression inside a larger expression (e.g., `WITH n { .name } AS info`) must parse correctly.
- Inline WHERE with a complex predicate (including nested subquery expressions) must not break the enclosing pattern parse.
- Quantifier `{0,}` (no upper bound) is valid in the BNF and must be accepted.
- GQL path search prefixes combined with path variable assignment (`MATCH p = ANY SHORTEST ...`) must parse correctly.
- `YIELD *` with and without a trailing WHERE must both parse correctly.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The grammar MUST parse `shortestPath(<path pattern>)` expressions and produce a `legacy_shortest_path_pattern` node.
- **FR-002**: The grammar MUST parse `allShortestPaths(<path pattern>)` expressions and produce a `legacy_shortest_path_pattern` node.
- **FR-003**: The grammar MUST allow a `WHERE <expression>` clause inside node patterns `(n WHERE ...)`.
- **FR-004**: The grammar MUST allow a `WHERE <expression>` clause inside relationship bodies `[r WHERE ...]`.
- **FR-005**: The grammar MUST parse map projection expressions of the form `variable { <projection elements> }` including field selectors (`.prop`), all-fields selector (`.*`), literal fields (`key: expr`), and variable selectors.
- **FR-006**: The grammar MUST parse GQL path search prefixes: `ALL`, `ANY [n]`, `SHORTEST n`, `ALL SHORTEST`, `ANY SHORTEST`, `SHORTEST n GROUPS` before a path pattern in a MATCH clause.
- **FR-007**: The grammar MUST parse quantified path patterns using `+`, `*`, `{n}`, and `{n,m}` quantifiers on parenthesized sub-paths.
- **FR-008**: The grammar MUST parse a `WHERE` clause following `YIELD` items in a `CALL` clause.
- **FR-009**: All new grammar rules MUST have corresponding corpus tests in `test/corpus/` that verify both successful parsing and correct node naming.
- **FR-010**: No existing corpus tests MUST regress after grammar changes.
- **FR-011**: The grammar MUST remain unambiguous after each rule addition — `tree-sitter generate` must complete without conflicts.

### Key Entities

- **Grammar rule**: A named production in `grammar.js` that maps to a BNF production. New rules must follow the naming convention `<bnf rule name>` → `snake_case`.
- **Corpus test**: A `.txt` file in `test/corpus/` with input Cypher and expected s-expression output, used to validate correct parsing.
- **Parse node**: A named node in the syntax tree produced by the parser. Consumers (lint tools, highlighters) navigate the tree by node name.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All six queries in the example `cypher/` directory parse with zero ERROR nodes.
- **SC-002**: The ten highest-impact gaps identified in `proposals/bnf-gap-analysis.md` (Notable Gaps section) are all addressed and covered by corpus tests.
- **SC-003**: `tree-sitter test` passes with 100% of existing tests green after all grammar changes are applied.
- **SC-004**: Each new grammar rule is covered by at least two corpus tests (happy path and at least one edge case).
- **SC-005**: `tree-sitter generate` produces no shift/reduce or reduce/reduce conflicts.

## Assumptions

- The BNF gap analysis document (`proposals/bnf-gap-analysis.md`) is the definitive prioritised list of missing rules; this spec addresses the "Notable Gaps" section (items 1–10) plus the partial-rule fixes listed in "Partial / Stub Rules."
- Purely structural wrapper rules that are already merged into richer parent rules (e.g., intermediate expression layers) do not need to be introduced as separate named nodes — only rules whose absence causes parse failures or incorrect tree structure are in scope.
- Tokenization helpers (character-level BNF rules) are implemented as regex terminals in Tree-sitter and are out of scope for named rule additions.
- CREATE/MERGE dedicated pattern sub-rules (which add semantic constraints rather than syntactic new forms) are out of scope for this feature; the grammar continues to reuse the general `pattern` rule for those clauses.
- Float type suffixes (`f`, `d`, `F`, `D`) and underscore digit separators in integer/float literals are in scope as small, low-risk additions.
- The `INF`, `INFINITY`, and `NAN` literal keywords are in scope and will be added as named literal nodes (not treated as plain identifiers).
- Grammar changes target the openCypher spec; Neo4j-specific extensions beyond what the spec defines are out of scope for this feature.
