# Feature Specification: Cypherdoc Injection Grammar

**Feature Branch**: `003-cypherdoc-grammar`
**Created**: 2026-05-07
**Status**: Draft
**Input**: User description: "Cypherdoc grammar as described in proposals/cypherdoc-injection.md"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Parse a named, documented Cypher tool (Priority: P1)

A developer writes a `/** ... */` doc comment above a Cypher statement, naming the query
and describing its parameters and return shape using cypherdoc tags. A Tree-sitter-aware
tool (editor, linter, code generator) parses the file and can extract the tool name,
param names/types/optionality, and the return tuple shape from the AST.

**Why this priority**: This is the core value of the feature — structured, machine-readable
documentation attached to Cypher statements. Everything else builds on this.

**Independent Test**: Can be tested by writing a `.cypher` file with a cypherdoc comment,
running `tree-sitter parse`, and confirming the AST contains `doc_comment > name`,
`param_tag`, and `returns_tag` nodes with correct structure.

**Acceptance Scenarios**:

1. **Given** a `.cypher` file with a `/** */` doc comment containing a name, `@param`, and
   `@returns`, **When** the file is parsed, **Then** the `doc_comment` node contains
   `name`, `param_tag`, and `returns_tag` child nodes with correctly typed content.
2. **Given** a required `@param {string} name - description`, **When** parsed, **Then** the
   AST contains a `required_param` node holding the identifier `name`.
3. **Given** an optional `@param {integer} [limit=25] - description`, **When** parsed,
   **Then** the AST contains an `optional_param` node with a `param_default` child
   holding `25`.

---

### User Story 2 - Describe return shape as a named tuple (Priority: P1)

A developer uses the `@returns` tag with TypeScript-style named tuple syntax to describe
the columns of the Cypher result set, including whether the query returns one row or many.

**Why this priority**: Tied with P1 — the return shape description is half of what makes a
cypherdoc comment useful for agent tool registration or API generation.

**Independent Test**: Parse a file containing `@returns {[person: node<Person>]}` and
`@returns {[name: string, age: integer][]}` and confirm the AST distinguishes the two via
presence/absence of `array_marker`.

**Acceptance Scenarios**:

1. **Given** `@returns {[person: node<Person>]}`, **When** parsed, **Then** the
   `returns_tag` contains a `tuple_type` with one `tuple_member` (`person: node<Person>`)
   and no `array_marker` node.
2. **Given** `@returns {[colleague_name: string, company: string][]}`, **When** parsed,
   **Then** the `returns_tag` contains a `tuple_type` with two `tuple_member` nodes and
   an `array_marker` node.
3. **Given** a `node<Person>` type in a tuple member, **When** parsed, **Then** the
   `scalar_type` node contains a `type_argument` child holding `Person`.

---

### User Story 3 - Inject cypherdoc into Cypher doc comments (Priority: P2)

An editor or tooling host that supports Tree-sitter language injection recognises `/** */`
blocks in `.cypher` files as `doc_comment` nodes and automatically parses their content
using the `cypherdoc` grammar, enabling cypherdoc-specific syntax highlighting and
symbol extraction without any manual configuration.

**Why this priority**: Depends on P1/P2 grammar correctness; no new grammar logic, just
the wiring between the two grammars.

**Independent Test**: Confirm `queries/injections.scm` in `tree-sitter-cypher` correctly
targets `doc_comment` nodes and sets `injection.language` to `"cypherdoc"`. Verify that
the cypherdoc grammar's `tree-sitter.json` registers the language name `cypherdoc`.

**Acceptance Scenarios**:

1. **Given** a `doc_comment` node in a parsed Cypher file, **When** the injections query
   is evaluated, **Then** the node is matched and tagged with language `"cypherdoc"`.
2. **Given** a cypherdoc grammar registered under the name `cypherdoc`, **When** an editor
   loads the injection, **Then** cypherdoc highlight captures apply inside `/** */` blocks.

---

### User Story 4 - Highlight cypherdoc syntax (Priority: P3)

An editor using `queries/highlights.scm` from `tree-sitter-cypherdoc` applies distinct
highlight groups to the tool name, tag keywords (`@param`, `@returns`), type annotations,
and descriptions, making doc comments visually distinct and easy to read.

**Why this priority**: Pure UX polish; the grammar must be correct first.

**Independent Test**: Run `tree-sitter highlight` on a `.cypher` file and confirm that
the name, tags, types, and descriptions each receive a distinct capture name.

**Acceptance Scenarios**:

1. **Given** a cypherdoc comment with all tag types, **When** highlights are applied,
   **Then** the tool name, `@param`/`@returns` keywords, type annotations, param names,
   and descriptions each receive a distinct `@` capture.

---

### Edge Cases

- A `/** */` block with no name line — should still parse without error (name is absent).
- A `/** */` block with only a name and no tags — valid minimal cypherdoc.
- A `@param` with a default value containing spaces or special characters (e.g., `[label="Person Node"]`).
- A `node<Label>` type argument containing multiple labels (e.g., `node<Person|Employee>`).
- A `tuple_type` with a single member vs. multiple members.
- Leading `*` decoration missing on some lines — grammar must still parse correctly.
- A `doc_comment` with multi-paragraph description before the first tag.
- Deeply nested list type: `list<list<string>>`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The cypherdoc grammar MUST parse `/** ... */` blocks including the delimiters
  and decorative ` * ` line prefixes.
- **FR-002**: The grammar MUST produce a `name` node from the first non-empty, non-tag
  content line.
- **FR-003**: The grammar MUST produce a `description` node from free prose appearing
  between the name and the first `@` tag.
- **FR-004**: The grammar MUST parse `@param {type} identifier - description` as a
  `param_tag` containing a `required_param` node.
- **FR-005**: The grammar MUST parse `@param {type} [identifier=default] - description`
  as a `param_tag` containing an `optional_param` node with a `param_default` child.
  A bare `[identifier]` without a default value MUST NOT be valid syntax.
- **FR-006**: The grammar MUST parse `@returns {[col: type, ...]}` as a `returns_tag`
  with a `tuple_type` and no `array_marker`.
- **FR-007**: The grammar MUST parse `@returns {[col: type, ...][]}` as a `returns_tag`
  with a `tuple_type` and an `array_marker` node.
- **FR-008**: The grammar MUST support all scalar types: `string`, `integer`, `float`,
  `boolean`, `node`, `node<Label>`, `relationship`, `relationship<TYPE>`, `path`,
  `list<type>`, `map`, `any`.
- **FR-009**: `node` and `relationship` types MUST support an optional `<Label>` or
  `<TYPE>` type argument.
- **FR-010**: The `queries/highlights.scm` MUST assign distinct captures to: tool name,
  tag keywords, type annotations, param/column identifiers, and tag descriptions.
- **FR-011**: `tree-sitter-cypher`'s `queries/injections.scm` MUST inject the `cypherdoc`
  language into all `doc_comment` nodes.

### Key Entities

- **`doc_comment`**: The root node produced by the cypherdoc grammar; represents the full
  `/** ... */` block.
- **`param_tag`**: Represents one `@param` entry; contains a type annotation, a
  `required_param` or `optional_param`, and a tag description.
- **`returns_tag`**: Represents the `@returns` entry; contains a type annotation (always
  a tuple type) and a tag description.
- **`tuple_member`**: A named column in the return tuple: `identifier: scalar_type`.
- **`scalar_type`**: A cypherdoc type name, optionally with a `type_argument`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All five example `.cypher` files in `cypher/` parse without errors or
  `ERROR` nodes in the `doc_comment` subtree.
- **SC-002**: The corpus test suite for `tree-sitter-cypherdoc` covers all tag forms,
  all scalar types, both tuple cardinalities, and required/optional params; all tests pass.
- **SC-003**: All existing `tree-sitter-cypher` corpus tests continue to pass after the
  injection wiring is in place.
- **SC-004**: A tool consuming the AST can extract tool name, all param names/types/
  defaults, and the return tuple shape from any of the five example files using only
  standard Tree-sitter query patterns — no string parsing of raw comment text required.

## Assumptions

- The cypherdoc grammar lives at `tree-sitter-cypherdoc/` within this repository and is
  developed as a self-contained Tree-sitter grammar subdirectory.
- The injection hook in `tree-sitter-cypher/queries/injections.scm` is already in place
  (it was added alongside the `doc_comment` rule).
- Default values in optional params are treated as opaque string literals by the grammar;
  semantic validation of defaults against types is out of scope.
- Multi-label type arguments (e.g., `node<Person|Employee>`) are desirable but deferred
  to a follow-on; the initial grammar supports single-label type arguments only.
- The grammar does not validate that `@param` names correspond to `$parameters` in the
  Cypher statement; that is a linter concern.
