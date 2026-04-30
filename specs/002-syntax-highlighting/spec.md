# Feature Specification: Cypher Syntax Highlighting and Code Navigation

**Feature Branch**: `002-syntax-highlighting`  
**Created**: 2026-04-29  
**Status**: Draft  
**Input**: User description: "cypher syntax highlighting and code navigation using highlights.scm locals.scm and tags.scm"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Syntax Highlighting (Priority: P1)

A developer opens a `.cypher` file in a tree-sitter-enabled editor (Neovim, Helix, Zed, etc.) and sees Cypher keywords, literals, operators, and identifiers rendered in distinct visual styles according to the editor's color theme.

**Why this priority**: Syntax highlighting is the most immediately visible benefit of a tree-sitter grammar. It requires only `highlights.scm` and delivers value as soon as the grammar parses any Cypher query. It is the foundation on which all other editor features build.

**Independent Test**: Can be fully tested by loading a sample Cypher query in an editor and observing that different token types receive distinct colors — no locals or tags support is required.

**Acceptance Scenarios**:

1. **Given** a Cypher query containing `MATCH`, `WHERE`, `RETURN`, `CREATE`, `MERGE`, `DELETE`, `SET`, `REMOVE`, `WITH`, `UNWIND`, `CALL`, `UNION`, `OPTIONAL`, `AS`, `AND`, `OR`, `NOT`, `IN`, `IS`, `NULL` keywords, **When** the file is opened in a tree-sitter-enabled editor, **Then** all keywords are highlighted using the `@keyword` capture group (or appropriate sub-group).

2. **Given** a query with string literals (`"hello"`, `'world'`), integer literals (`42`), float literals (`3.14`), boolean literals (`true`, `false`), and `null`, **When** the file is rendered, **Then** each literal type is highlighted distinctly: strings as `@string`, numbers as `@number`, booleans as `@boolean`, null as `@constant.builtin`.

3. **Given** a query with node labels (`(:Person)`), relationship types (`-[:KNOWS]->`), property keys (`n.name`), and function calls (`count(n)`, `toUpper(n.name)`), **When** rendered, **Then** labels and types appear as `@type`, property accesses appear appropriately styled, and function names appear as `@function`.

4. **Given** a query using arithmetic/comparison operators (`+`, `-`, `*`, `/`, `=`, `<>`, `<`, `>`, `<=`, `>=`), **When** rendered, **Then** they are highlighted as `@operator`. Keyword-spelled operators (`AND`, `OR`, `NOT`, `XOR`, `IN`, `IS`, `CONTAINS`, `STARTS WITH`, `ENDS WITH`) are highlighted as `@keyword.operator` to distinguish them visually as reserved words.

5. **Given** a query with comments (`// single line`), **When** rendered, **Then** comments are highlighted as `@comment`.

6. **Given** a query with named parameters (`$paramName`), **When** rendered, **Then** parameters are highlighted as `@variable.parameter`.

---

### User Story 2 - Local Scope and Variable Tracking (Priority: P2)

A developer navigating a Cypher query can see variable definitions and references highlighted consistently, enabling features like "rename symbol" and semantic highlighting in editors that support tree-sitter locals.

**Why this priority**: Local scope tracking enables semantic editor features beyond basic coloring. It requires `locals.scm` and depends on a working highlights.scm (P1). It is the next most impactful navigation feature.

**Independent Test**: Can be tested by verifying that a variable introduced in a `MATCH` clause (e.g., `n` in `MATCH (n:Person)`) is recognized as a definition, and subsequent references to `n` in `WHERE n.name = 'Alice'` and `RETURN n` are recognized as references to the same binding.

**Acceptance Scenarios**:

1. **Given** `MATCH (n:Person)-[:KNOWS]->(m:Person) RETURN n, m`, **When** a locals-aware editor analyzes the query, **Then** `n` and `m` node pattern identifiers are marked as `@local.definition` at their introduction site and `@local.reference` at each subsequent use.

2. **Given** a `WITH` clause that projects and renames variables (e.g., `WITH n AS person`), **When** analyzed, **Then** `person` is recognized as a new `@local.definition` in the scope following the `WITH`, and `n` before the `WITH` is a reference.

3. **Given** two separate Cypher statements separated by `;`, **When** analyzed by a locals-aware editor, **Then** variables from the first statement do not appear as references in the second statement.

---

### User Story 3 - Code Navigation Tags (Priority: P3)

A developer using ctags-compatible tools or editor "workspace symbols" search can navigate to definitions of named Cypher procedures, functions referenced in queries, and named parameters used across multiple query files.

**Why this priority**: Tags enable cross-file navigation and symbol search. They require `tags.scm` and are useful primarily in projects that store multiple Cypher files. Lower priority because value depends on multi-file workflows.

**Independent Test**: Can be tested by running a tree-sitter tags query against a Cypher file and verifying that procedure names (`CALL db.schema.visualization()`), user-defined function names, and named parameters produce tag entries recognized by navigation tools.

**Acceptance Scenarios**:

1. **Given** a Cypher file containing `CALL apoc.load.json($url)`, **When** tags are generated, **Then** `apoc.load.json` appears as a `@definition.function` tag entry.

2. *(Descoped)* Named parameters (`$userId`) are externally-supplied values with no definition site within a Cypher file. They are highlighted as `@variable.parameter` in `highlights.scm` but are not indexed as navigable tags; cross-file parameter navigation depends on the calling application context, not the query files.

3. **Given** multiple `.cypher` files in a project workspace, **When** a developer searches for a symbol, **Then** the tags index allows jumping to the relevant query file.

---

### Edge Cases

- What happens when a Cypher query is syntactically invalid — does the partial parse still produce useful highlights for the valid portions?
- How does the system handle case-insensitive keywords (`MATCH` vs `match` vs `Match`) — are all variants highlighted identically?
- What happens with escaped identifiers using backticks (`` `My Label` ``) — are they highlighted as identifiers or labels?
- How does highlights.scm handle ambiguous contexts where an identifier could be a property key, a variable, or a label?
- What happens with Cypher queries embedded in other languages (e.g., JavaScript strings calling Neo4j) — does the injections.scm interact with highlights?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `highlights.scm` MUST assign `@keyword` captures to all Cypher clause keywords (`MATCH`, `RETURN`, `CREATE`, `MERGE`, `DELETE`, `SET`, `REMOVE`, `WITH`, `UNWIND`, `CALL`, `UNION`, `OPTIONAL`, `WHERE`, `ORDER BY`, `SKIP`, `LIMIT`, `DISTINCT`).
- **FR-002**: `highlights.scm` MUST assign `@keyword.operator` or `@operator` captures to logical operators (`AND`, `OR`, `NOT`, `XOR`, `IN`, `IS`, `CONTAINS`, `STARTS WITH`, `ENDS WITH`).
- **FR-003**: `highlights.scm` MUST assign `@string` to string literals, `@number` to integer and float literals, `@boolean` to boolean literals, and `@constant.builtin` to `null`.
- **FR-004**: `highlights.scm` MUST assign `@type` captures to node labels and relationship type names in pattern expressions.
- **FR-005**: `highlights.scm` MUST assign `@function` captures to built-in and user-defined function names and procedure names.
- **FR-006**: `highlights.scm` MUST assign `@variable.parameter` to named parameters (e.g., `$param`).
- **FR-007**: `highlights.scm` MUST assign `@comment` to both single-line (`//`) and block (`/* ... */`) comments.
- **FR-008**: `highlights.scm` MUST assign `@operator` to arithmetic, comparison, and string operators.
- **FR-009**: `highlights.scm` MUST assign `@punctuation.delimiter` to commas, semicolons, and dots; `@punctuation.bracket` to parentheses, brackets, and braces.
- **FR-010**: `locals.scm` MUST mark variable introductions in node and relationship patterns as `@local.definition`.
- **FR-011**: `locals.scm` MUST mark subsequent uses of those variables as `@local.reference`.
- **FR-012**: `locals.scm` MUST define scope boundaries at the `statement` and `union_statement` level so that variables in different query statements do not bleed across boundaries. Full within-statement WITH and CALL {} sub-scoping is deferred; the conservative model is documented in `specs/002-syntax-highlighting/research.md` section 3.
- **FR-013**: `tags.scm` MUST emit `@definition.function` tags for procedure and function names encountered in `CALL` clauses.
- **FR-014**: All capture names MUST conform to the standard tree-sitter highlight name taxonomy to ensure compatibility with editors that use tree-sitter (e.g., Neovim, Helix, Zed).

### Key Entities

- **Highlight Capture**: A named group (e.g., `@keyword`, `@string`) assigned to an AST node, consumed by the editor's theme engine.
- **Local Definition**: A variable binding site where a new name enters scope (node/relationship pattern identifiers, `AS` aliases).
- **Local Reference**: A use site of a previously defined variable.
- **Scope**: A region of a query where a set of variable definitions is valid (bounded by `WITH`, `UNION`, or subquery boundaries).
- **Tag**: A navigable symbol entry (procedure name, function name) that can be indexed across files.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of Cypher clause keywords in sample queries receive an appropriate `@keyword` highlight capture — verified by running the highlights query against a representative test suite of Cypher queries.
- **SC-002**: All literal types (string, integer, float, boolean, null) in sample queries receive distinct, correctly categorized captures — zero mis-categorized literals in the test suite.
- **SC-003**: Variable definitions and references in standard MATCH/WITH/RETURN patterns are correctly linked by locals — a locals-aware editor can perform "rename symbol" on a variable and all occurrences update correctly.
- **SC-004**: Tags generated from a set of Cypher files containing procedure calls are parseable by ctags-compatible tools and produce navigable entries.
- **SC-005**: The highlights, locals, and tags queries parse without errors against the current grammar's node types — verified by running `tree-sitter query` validation with no reported errors.
- **SC-006**: Partial/invalid Cypher queries still produce meaningful highlights for their valid sub-trees — the query files handle ERROR nodes gracefully.

## Assumptions

- The `grammar.js` `kw()` helper must be updated to wrap keyword tokens with `alias(..., str.toLowerCase())` so that keywords appear as anonymous nodes in the AST; this is a prerequisite for keyword highlighting and is covered in Phase 2 of the implementation plan.
- Editors consuming these query files support the standard tree-sitter highlight name taxonomy; non-standard capture names are avoided.
- `locals.scm` scope modeling follows the Cypher semantics where `WITH` acts as a scope boundary — variables introduced before a `WITH` must be explicitly projected to remain in scope after it.
- Named parameters (`$param`) are treated as variable references to externally-provided values; they do not have local definition sites within the query.
- The `injections.scm` file (already present) is out of scope for this feature; only `highlights.scm`, `locals.scm`, and `tags.scm` are in scope.
- Multi-file workspace symbol search via `tags.scm` depends on the host editor's tags indexing capability — this feature only provides correct query content, not editor configuration.
