# Feature Specification: Expand Lint Coverage with Neo4j Notification Codes

**Feature Branch**: `007-expand-lint-notifications`
**Created**: 2026-05-10
**Status**: Draft
**Input**: User description: "Expand lint coverage for the cypher cli tool/lib by reviewing the Neo4j list of notification codes provided by Neo4j."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Detect Cartesian Product Patterns (Priority: P1)

A developer writes a MATCH clause with two disconnected node patterns — no relationship or path connecting them. Without a join condition this produces a full cross-product of the two result sets, which can be catastrophically expensive on large graphs. The linter flags it before the query is ever run.

**Why this priority**: Cartesian products are among the most common accidental performance killers in Cypher. Neo4j's own notification system (code 03N90) calls this out at INFORMATION level; the linter should catch it statically before execution.

**Independent Test**: Run `cypher lint -e "MATCH (a:User), (b:Order) RETURN a, b"`. A warning diagnostic with rule `CartesianProduct` is emitted pointing to the disconnected pattern. A query connecting the two with a relationship emits no diagnostic.

**Acceptance Scenarios**:

1. **Given** a Cypher query with `MATCH (a:A), (b:B)` and no relationship between `a` and `b`, **When** linted, **Then** a `CartesianProduct` warning is reported at the second disconnected pattern.
2. **Given** a query with `MATCH (a:A)-[:R]->(b:B), (c:C)` where `c` is unconnected, **When** linted, **Then** a `CartesianProduct` warning is reported for `c`.
3. **Given** a query with a single MATCH pattern, **When** linted, **Then** no cartesian-product diagnostic is emitted.
4. **Given** a query with `MATCH (a), (b) WHERE id(a) = id(b)`, **When** linted, **Then** a `CartesianProduct` warning is still reported (static analysis cannot evaluate WHERE predicates).

---

### User Story 2 - Flag Deprecated `id()` Function (Priority: P1)

A developer uses the `id()` function to retrieve a node's internal identifier. Neo4j 5 deprecated `id()` in favour of `elementId()`, which returns a stable string identifier. The linter warns at the call site so the developer can migrate before the function is removed.

**Why this priority**: This deprecation (code 01N01) is live in Neo4j 5 and affects any team upgrading. Catching it at write-time prevents surprise breakage.

**Independent Test**: Run `cypher lint -e "MATCH (n) RETURN id(n)"`. A `DeprecatedFunction` warning is emitted pointing to `id(`. A query using `elementId(n)` emits no diagnostic.

**Acceptance Scenarios**:

1. **Given** `RETURN id(n)`, **When** linted, **Then** a `DeprecatedFunction` warning is emitted with a message that suggests `elementId()` as the replacement.
2. **Given** `RETURN elementId(n)`, **When** linted, **Then** no deprecation diagnostic is emitted.
3. **Given** `MATCH (n) WHERE id(n) > 0 RETURN n`, **When** linted, **Then** the warning points to the `id(` call in the WHERE clause.
4. **Given** `RETURN id(r)` on a relationship, **When** linted, **Then** the same rule fires.

---

### User Story 3 - Flag Dynamic Property Access (Priority: P2)

A developer accesses a property with a variable key: `n[$prop]` or `n[someExpr]`. This pattern prevents the query planner from using indexes, because the property name is not known until runtime. The linter flags it as a potential performance concern.

**Why this priority**: Neo4j's notification code 03N95 highlights this pattern. It is detectable purely from AST shape and is a common source of unexpected slow queries.

**Independent Test**: Run `cypher lint -e "MATCH (n) WHERE n[$key] IS NOT NULL RETURN n"`. A `DynamicProperty` information diagnostic is emitted at the bracketed access. A query using `n.name` emits no diagnostic.

**Acceptance Scenarios**:

1. **Given** `n[$prop]` in a WHERE clause, **When** linted, **Then** a `DynamicProperty` information diagnostic is reported at the dynamic key.
2. **Given** `n.name` (static property access), **When** linted, **Then** no diagnostic is emitted.
3. **Given** `n[expr()]` where the key is a function call, **When** linted, **Then** a `DynamicProperty` diagnostic is emitted.
4. **Given** `SET n[$key] = 1`, **When** linted, **Then** the same rule fires for the write side as well.

---

### User Story 4 - Flag Deprecated Colon-Separated Relationship Types (Priority: P2)

A developer writes relationship type alternatives using the old colon-separated syntax: `[:A|:B|:C]`. In modern Cypher this should be `[:A|B|C]` (no leading colons after the first). The linter warns at each occurrence so the developer can update the syntax before it is removed.

**Why this priority**: Code 01N01 lists this as a named deprecation. It is syntactically unambiguous and trivially detectable from the AST.

**Independent Test**: Run `cypher lint -e "MATCH (a)-[:FOO|:BAR]->(b) RETURN a"`. A `DeprecatedRelationshipTypeList` warning is emitted. The modern form `[:FOO|BAR]` emits no diagnostic.

**Acceptance Scenarios**:

1. **Given** `[:A|:B]`, **When** linted, **Then** a `DeprecatedRelationshipTypeList` warning is emitted.
2. **Given** `[:A|B]`, **When** linted, **Then** no diagnostic is emitted.
3. **Given** `[:A|:B|:C]` (three alternatives with extra colons), **When** linted, **Then** a single diagnostic is emitted covering the pattern.

---

### User Story 5 - Diagnostic Codes Align with Neo4j Notification Codes (Priority: P2)

Every new lint rule includes a `code` field in its diagnostic output that references the corresponding Neo4j notification code (e.g., `03N90`, `01N01`, `03N95`). This lets tooling link directly to the official Neo4j documentation and lets users understand the origin of a warning.

**Why this priority**: The `Diagnostic` type already has a `code: Option<String>` field that is unused. Populating it with official Neo4j notification codes turns the linter into a locally-executable mirror of Neo4j's own notification system, which increases its authority and usability.

**Independent Test**: Run `cypher lint --json -e "MATCH (a:A), (b:B) RETURN a, b"`. The JSON output for the `CartesianProduct` diagnostic has `"code": "03N90"`.

**Acceptance Scenarios**:

1. **Given** any new rule fires, **When** `--json` output is inspected, **Then** the `code` field is non-null and matches the corresponding Neo4j notification code.
2. **Given** human-readable (`--pretty`) output, **When** a new rule fires, **Then** the rule code shown in the report matches the Neo4j notification code.

---

### Edge Cases

- What happens when a MATCH clause has three or more disconnected patterns? Each additional disconnected pattern should produce its own `CartesianProduct` diagnostic.
- What happens when `id()` appears inside a list comprehension or subquery? The rule should still fire wherever the call appears in the AST.
- What if a query uses both `id()` and `elementId()`? Only the `id()` call is flagged; `elementId()` is clean.
- What if a dynamic property access is used in a WITH or RETURN clause rather than WHERE? The `DynamicProperty` rule applies regardless of clause context.
- What about `shortestPath` with a fixed-length relationship (deprecated per 01N01)? This is deferred — it overlaps with the existing `UnboundedRelationship` rule and requires careful grammar inspection; it is out of scope for this feature.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The linter MUST detect disconnected MATCH patterns and emit a `CartesianProduct` warning (Neo4j code 03N90).
- **FR-002**: The linter MUST detect calls to the `id()` function and emit a `DeprecatedFunction` warning with a message suggesting `elementId()` (Neo4j code 01N01).
- **FR-003**: The linter MUST detect dynamic property access (`n[expr]` where `expr` is not a string literal) and emit a `DynamicProperty` information diagnostic (Neo4j code 03N95).
- **FR-004**: ~~`DeprecatedRelationshipTypeList`~~ **Deferred** — `[:A|:B]` already produces a `ParseError` in the current grammar (a MISSING node is inserted at the unexpected `:`, caught by the existing `collect_error_nodes` path). A dedicated lint rule would be redundant; a future feature can improve the error message specificity.
- **FR-005**: Every new rule's diagnostic MUST populate the `code` field with the corresponding Neo4j notification code string.
- **FR-006**: All new rules MUST be implemented as `.scm` rule files following the existing `Rule: / Severity: / Applies-to: / Message:` header convention, loadable via the existing `--rules-dir` mechanism as well as compiled in as builtins.
- **FR-007**: Each new rule MUST be suppressible via the existing `--rule <name>` filter (i.e., users can run only a subset of rules).
- **FR-008**: New rules MUST fire consistently whether input comes from a `.cypher` file, a markdown fenced block, or `--expression`.

### Key Entities

- **Rule**: A `.scm` file with metadata headers and a tree-sitter query; identified by a string name aligned with the Neo4j notification concept.
- **Diagnostic**: The output record emitted for a rule match; includes `severity`, `rule` name, `message`, source `range`, and `code` (Neo4j notification code).
- **Notification Code**: A Neo4j-defined alphanumeric identifier (e.g., `03N90`) that categorises a class of query issue; used as the `code` value in diagnostics.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Three new lint rules are available out-of-the-box (`CartesianProduct`, `DeprecatedFunction`, `DynamicProperty`). (`DeprecatedRelationshipTypeList` was deferred — see FR-004.)
- **SC-002**: Each new rule has at least one integration test asserting the rule fires (positive case) and at least one asserting it does not fire (negative/clean case), all passing with `cargo test` in `tools/cypher/tests/lint_integration.rs`.
- **SC-003**: `cypher lint --json` output for every new rule includes a non-null `code` field containing the correct Neo4j notification code.
- **SC-004**: All existing lint tests continue to pass after the new rules are added (no regressions).
- **SC-005**: A developer encountering an unfamiliar diagnostic can look up the `code` value in the Neo4j notification reference and immediately understand its origin.

## Assumptions

- The existing tree-sitter grammar already parses all AST nodes needed to express the new queries (disconnected patterns, function calls, property access, relationship type lists); no grammar changes are required.
- Rules are limited to patterns detectable from the static AST alone — those requiring live database schema (unknown label 01N50, unknown relationship type 01N51, unknown property key 01N52) are out of scope.
- The `Diagnostic.code` field is already present in the type definition and serialised in `--json` output; populating it requires no structural changes to the output format.
- Severity levels follow Neo4j's notification severity where reasonable: PERFORMANCE/INFORMATION → `Information`, DEPRECATION/WARNING → `Warning`.
- The `UnboundedRelationship` rule (already shipping) covers Neo4j notification 03N91; it is not re-implemented here.
