# Feature Specification: Port openCypher BNF to Tree-sitter Grammar

**Feature Branch**: `001-port-opencypher-bnf`
**Created**: 2026-04-29
**Status**: Draft
**Input**: User description: "Port the openCypher BNF to a tree-sitter grammar"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Literals, Identifiers, and Comments (Priority: P1)

A developer needs the parser to correctly recognize the atomic building blocks of Cypher: all literal value types, identifiers, keywords, and comments.

**Why this priority**: Everything else in the grammar builds on these terminal rules. A bug here breaks every higher-level story. This slice can be verified with zero Cypher clause knowledge.

**Independent Test**: Can be fully tested by parsing isolated literal expressions and identifiers and verifying each produces a correctly typed tree node with no ERROR nodes.

**Acceptance Scenarios**:

1. **Given** integer, float, hex, octal, string, boolean, and null literals, **When** parsed, **Then** each produces a distinct typed leaf node.
2. **Given** a string with escape sequences (e.g., `"line\nbreak"`), **When** parsed, **Then** the string node captures the full content including escapes.
3. **Given** a line comment `// text` or block comment `/* text */`, **When** parsed, **Then** the comment is skipped without affecting surrounding nodes.
4. **Given** a keyword used as a clause name (e.g., `MATCH`, `match`, `Match`), **When** parsed, **Then** it is recognized case-insensitively.

---

### User Story 2 - Minimal MATCH/RETURN (Priority: P2)

A developer needs the parser to handle the simplest complete Cypher query — match a node and return it — producing a structured tree that identifies the clause boundaries.

**Why this priority**: `MATCH (n) RETURN n` is the "hello world" of Cypher. A passing parse here proves the top-level statement structure, clause sequencing, and basic node pattern all work together.

**Independent Test**: Can be fully tested by parsing `MATCH (n) RETURN n` and variants, verifying the tree identifies the statement, MATCH clause, node pattern, and RETURN clause as distinct named nodes.

**Acceptance Scenarios**:

1. **Given** `MATCH (n) RETURN n`, **When** parsed, **Then** the tree contains a statement with a MATCH clause, a node pattern with variable `n`, and a RETURN clause.
2. **Given** `MATCH (n) RETURN n.name, n.age`, **When** parsed, **Then** the RETURN clause lists two property access expressions.
3. **Given** `MATCH (n) WHERE n.active = true RETURN n`, **When** parsed, **Then** the WHERE clause with a boolean comparison is represented in the tree.

---

### User Story 3 - Graph Patterns (Priority: P3)

A developer building a graph query analyzer needs the parser to distinguish all pattern shapes: node patterns with labels and properties, relationship patterns with type and direction, path variables, and variable-length ranges.

**Why this priority**: Graph patterns are the defining feature of Cypher. This story extends P2's minimal node pattern to the full range of structural descriptions the language supports.

**Independent Test**: Can be fully tested by parsing MATCH clauses with varied pattern shapes and verifying the AST correctly distinguishes node patterns, relationship patterns, direction, labels, types, and quantifiers.

**Acceptance Scenarios**:

1. **Given** `(n:Person {name: 'Alice'})`, **When** parsed, **Then** the node pattern identifies the variable, label, and property map as distinct child nodes.
2. **Given** `(a)-[r:KNOWS]->(b)`, **When** parsed, **Then** the relationship pattern identifies direction, variable, and type.
3. **Given** `(a)-[:KNOWS*1..3]->(b)`, **When** parsed, **Then** the range quantifier `1..3` is represented in the tree.
4. **Given** `p = (a)-[*]->(b)`, **When** parsed, **Then** the path variable `p` is associated with the pattern.

---

### User Story 4 - Expressions and WHERE (Priority: P4)

A developer building a Cypher formatter or linter needs the parser to decompose expressions into structured nodes that reflect operator precedence, function calls, and all compound value forms.

**Why this priority**: Expressions appear inside WHERE, RETURN, SET, and WITH clauses. Getting operator precedence and nesting right is necessary before data-mutation and pipeline clauses can be fully implemented.

**Independent Test**: Can be fully tested by parsing standalone expressions in WHERE clauses and verifying arithmetic, comparison, boolean, list, map, and function-call forms each produce correctly nested tree nodes.

**Acceptance Scenarios**:

1. **Given** `WHERE a.age > 18 AND a.active = true`, **When** parsed, **Then** the AND expression nests the two comparisons as children with correct precedence.
2. **Given** a property access `n.name`, **When** parsed, **Then** the variable and property name are distinct child nodes.
3. **Given** a list literal `[1, 2, 3]` and map literal `{key: 'val'}`, **When** parsed, **Then** each element is a separate child node.
4. **Given** a function call `toUpper(n.name)`, **When** parsed, **Then** the function name and argument are identified as distinct nodes.

---

### User Story 5 - Data Mutation Clauses (Priority: P5)

A developer building a schema migration tool needs the parser to correctly structure CREATE, SET, REMOVE, and DELETE clauses, including DETACH DELETE and all SET item forms.

**Why this priority**: Mutation clauses are the write path of Cypher. They depend on graph patterns (P3) and expressions (P4) being in place, but are otherwise independent of pipeline and advanced clauses.

**Independent Test**: Can be fully tested by parsing queries that only contain mutation clauses and verifying each clause type and its sub-elements (set items, remove items, delete targets) are correctly identified.

**Acceptance Scenarios**:

1. **Given** `CREATE (n:Person {name: 'Alice'})`, **When** parsed, **Then** the CREATE clause contains a node pattern with label and property map.
2. **Given** `SET n.age = 30, n += {active: true}`, **When** parsed, **Then** the two set items (property assignment and map merge) are distinct nodes.
3. **Given** `DETACH DELETE n`, **When** parsed, **Then** the DETACH modifier is captured alongside the DELETE clause.
4. **Given** `REMOVE n:Label, n.prop`, **When** parsed, **Then** the label-remove and property-remove items are identified separately.

---

### User Story 6 - Pipeline Clauses (Priority: P6)

A developer building a Cypher query planner needs the parser to handle WITH, UNWIND, and ORDER BY/SKIP/LIMIT so multi-step pipelines can be analyzed as a sequence of named stages.

**Why this priority**: Pipeline clauses enable multi-hop query patterns. They require expression support (P4) but are independent of mutation (P5) and advanced features.

**Independent Test**: Can be fully tested by parsing multi-clause queries using WITH and UNWIND and verifying the clause sequence, projection list, ORDER BY, and paging elements are correctly represented.

**Acceptance Scenarios**:

1. **Given** `MATCH (n) WITH n ORDER BY n.name SKIP 10 LIMIT 5 RETURN n`, **When** parsed, **Then** WITH, ORDER BY, SKIP, and LIMIT are represented as structured nodes.
2. **Given** `UNWIND [1,2,3] AS x RETURN x`, **When** parsed, **Then** the UNWIND clause identifies the list expression and binding variable.
3. **Given** `MATCH (n) WITH n WHERE n.active = true RETURN n`, **When** parsed, **Then** the WHERE clause on the WITH is attached to the correct clause.

---

### User Story 7 - MERGE and CALL (Priority: P7)

A developer building an ETL pipeline tool needs the parser to handle MERGE with ON MATCH/ON CREATE actions and CALL for procedure invocation with YIELD.

**Why this priority**: MERGE and CALL are the remaining core clause types. They depend on graph patterns and expressions but have distinct semantics that require dedicated grammar rules.

**Independent Test**: Can be fully tested by parsing queries containing only MERGE or CALL clauses and verifying merge actions and procedure argument/yield lists are correctly structured.

**Acceptance Scenarios**:

1. **Given** `MERGE (n:Person {id: 1}) ON CREATE SET n.name = 'Alice' ON MATCH SET n.seen = true`, **When** parsed, **Then** both ON CREATE and ON MATCH actions are represented as children of the MERGE clause.
2. **Given** `CALL db.labels() YIELD label RETURN label`, **When** parsed, **Then** the procedure name, argument list, and YIELD items are identified as distinct nodes.
3. **Given** a standalone `CALL db.labels()` (without YIELD), **When** parsed, **Then** it is accepted as a valid standalone procedure call.

---

### User Story 8 - UNION and Advanced Expressions (Priority: P8)

A developer building a Cypher optimizer needs the parser to handle UNION/UNION ALL for combining result sets, and advanced expressions including CASE, list comprehensions, and pattern predicates.

**Why this priority**: These are the most complex and least commonly used constructs. They require all prior stories to be in place and complete the full openCypher BNF coverage.

**Independent Test**: Can be fully tested by parsing queries that use UNION and advanced expression forms, verifying each composite structure is correctly nested without ERROR nodes.

**Acceptance Scenarios**:

1. **Given** `MATCH (n:A) RETURN n UNION MATCH (n:B) RETURN n`, **When** parsed, **Then** the two linear statements are identified as children of a UNION expression.
2. **Given** `CASE n.status WHEN 'active' THEN 1 ELSE 0 END`, **When** parsed, **Then** the CASE expression, WHEN/THEN pairs, and ELSE branch are distinct nodes.
3. **Given** a list comprehension `[x IN list WHERE x > 0 | x * 2]`, **When** parsed, **Then** the variable, source, filter, and projection are identified separately.

---

### Edge Cases

- Queries with no whitespace between tokens (minimal spacing)
- Queries with Unicode identifiers and string values
- Deeply nested expressions and subqueries
- Case-insensitive keywords (Cypher keywords are case-insensitive: `MATCH`, `match`, `Match` are equivalent)
- String literals with escaped characters
- Comments (single-line `//` and multi-line `/* */`)
- The full range of numeric literals (integers, floats, hex `0x`, octal `0o`)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The grammar MUST cover all BNF rules in `references/openCypher/grammar/openCypher.bnf`
- **FR-002**: The grammar MUST treat Cypher keywords as case-insensitive (e.g., `MATCH`, `match`, `Match` all parse identically)
- **FR-003**: The grammar MUST parse all statement types: MATCH, OPTIONAL MATCH, CREATE, MERGE, SET, REMOVE, DELETE, WITH, UNWIND, RETURN, CALL
- **FR-004**: The grammar MUST parse graph patterns including node patterns, directed and undirected relationship patterns, path variables, label expressions, and variable-length ranges
- **FR-005**: The grammar MUST parse all expression types: arithmetic, comparison, boolean, string, list, map, property access, function calls, and CASE expressions
- **FR-006**: The grammar MUST parse all literal types: integer, float, hexadecimal, octal, string (with escape sequences), boolean, null, list, and map
- **FR-007**: The grammar MUST parse UNION and UNION ALL for combining query results
- **FR-008**: The grammar MUST parse CALL for procedure invocation including YIELD clauses
- **FR-009**: The grammar MUST skip line comments (`//`) and block comments (`/* */`) as extras
- **FR-010**: Corpus tests MUST cover every statement type and major expression variant
- **FR-011**: `tree-sitter test` MUST pass with zero failures after implementation

### Key Entities

- **Statement**: A top-level query unit; composite (UNION) or linear (clause sequence)
- **Clause**: A named query step (MATCH, WITH, RETURN, CREATE, etc.)
- **Graph Pattern**: A structural description of nodes and relationships to match or create
- **Node Pattern**: A single vertex in a pattern, optionally with variable, labels, and properties
- **Relationship Pattern**: An edge in a pattern, with optional variable, type, properties, and direction
- **Expression**: A value-producing syntactic form (literal, variable, operator, function call)
- **Label Expression**: A boolean combination of node labels using `|`, `&`, `!`, `%`

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `tree-sitter parse` produces zero ERROR nodes when parsing any query in the openCypher TCK feature corpus at `references/openCypher/tck/`
- **SC-002**: `tree-sitter test` passes 100% of corpus tests with no failures
- **SC-003**: All 12 top-level BNF sections in `openCypher.bnf` are covered by grammar rules and tested
- **SC-004**: Parsing a 100-line Cypher file completes in under 50ms (incremental re-parse of a single changed line completes in under 5ms)
- **SC-005**: Every grammar rule has at least one passing corpus test

## Assumptions

- The grammar targets the openCypher specification as defined in `references/openCypher/grammar/openCypher.bnf` (not Neo4j-specific extensions like GQL or Cypher 25)
- Cypher keywords are reserved and cannot be used as unquoted identifiers
- The tree-sitter grammar will use named nodes for all meaningful syntactic elements (not anonymous for clause keywords)
- The implementation proceeds in 8 incremental slices matching the user stories: literals → minimal MATCH/RETURN → graph patterns → expressions → mutation clauses → pipeline clauses → MERGE/CALL → UNION and advanced expressions
- Existing language bindings in `bindings/` do not need modification — they derive automatically from the generated parser
