# Research: Cypher Syntax Highlighting and Code Navigation

**Feature**: 002-syntax-highlighting | **Date**: 2026-04-29

---

## 1. Keyword Node Exposure in Tree-sitter Grammars

### Finding

The current `kw()` helper creates `token(new RegExp(...))` — a regex-based anonymous terminal. Parsing `MATCH (n:Person) RETURN n` with `tree-sitter parse --xml` confirms that **no AST nodes exist for `MATCH`, `RETURN`, or any other keyword**. Only string-literal anonymous tokens (`(`, `:`, `)`) and named rules (`identifier`, `label_expression`, etc.) appear.

Anonymous node types from `src/node-types.json` include only operator/punctuation tokens — zero keyword types.

### Decision

**Use `alias()` to assign each keyword occurrence a canonical lowercase anonymous node type.**

Change `kw('MATCH')` → `alias(token(/[Mm][Aa][Tt][Cc][Hh]/), 'match')` at each use site.

This creates anonymous nodes of type `'match'`, `'return'`, etc. in the AST. Query files then capture them with `"match" @keyword`.

- Rationale: Minimal grammar diff. No new named rules; no changes to conflict resolution or precedence. The canonical lowercase type matches tree-sitter convention for anonymous keyword nodes (same pattern used by tree-sitter-sql, tree-sitter-lua, etc.).
- Alternatives considered:
  - **Named keyword rules** (`kw_match: _ => /[Mm][Aa][Tt][Cc][Hh]/`): Works but requires renaming every use site (`$.kw_match` instead of `kw('MATCH')`) and adds 30+ new rule names to node-types.json.
  - **`word` property + keyword extraction**: The standard tree-sitter approach for reserved word handling; requires a named identifier token and careful conflict setup. Higher complexity than alias approach for a grammar already using regex tokens.
  - **Clause-level captures only**: Capture `(match_clause) @keyword` for the clause container — highlights the entire clause, not just the keyword. Not viable.

### Implementation Note

Update `kw()` helper to accept an optional `alias_as` parameter, or replace inline `kw('X')` calls with `alias(kw('X'), 'x')` at each use site. The helper can be updated to:

```js
const kw = (str, as_alias = null) => {
  const token_ = token(new RegExp(
    str.split('').map(c =>
      /[a-zA-Z]/.test(c) ? `[${c.toUpperCase()}${c.toLowerCase()}]` : c
    ).join('')
  ));
  return as_alias ? alias(token_, as_alias) : token_;
};
```

Then each keyword in a rule becomes `kw('MATCH', 'match')`, producing a node of type `'match'` that can be queried as `"match" @keyword`.

**Constitution gates for this change**:
- Fidelity: Each keyword terminal maps to a BNF reserved word — passes.
- Dual-coverage: Existing positive tests already exercise these keywords; add negative tests (e.g., missing keyword → ERROR). No new rules added.
- TCK: Change is token-level only; parse tree shape is unchanged for named rules. Full TCK re-run required to confirm zero regressions.

---

## 2. Standard Tree-sitter Highlight Capture Taxonomy

### Finding

Tree-sitter's [highlight name spec](https://tree-sitter.github.io/tree-sitter/syntax-highlighting) defines a hierarchy of capture group names. Editors that consume highlights.scm map these to color theme variables. The most relevant groups for Cypher:

| Capture | Used for |
|---------|----------|
| `@keyword` | Clause keywords: MATCH, RETURN, CREATE, MERGE, DELETE, SET, REMOVE, WITH, UNWIND, CALL, UNION |
| `@keyword.operator` | Logical/predicate operators spelled as words: AND, OR, NOT, XOR, IN, IS, CONTAINS, STARTS WITH, ENDS WITH |
| `@keyword.control` | Flow modifiers: OPTIONAL, DISTINCT, DETACH |
| `@keyword.modifier` | Modifier keywords: ASC, DESC, ALL, ANY, NONE, SINGLE, REDUCE |
| `@string` | String literals |
| `@number` | Integer and float literals |
| `@boolean` | Boolean literals (true, false) |
| `@constant.builtin` | null |
| `@type` | Node labels and relationship types |
| `@function` | Function call names and procedure names |
| `@variable` | Graph pattern variables (node/rel identifiers) |
| `@variable.parameter` | Named parameters ($param) |
| `@operator` | Arithmetic/comparison operators: +, -, *, /, =, <>, <, >, <=, >=, =~, ^ |
| `@comment` | Comments (// and /* */) |
| `@punctuation.delimiter` | , ; . |
| `@punctuation.bracket` | ( ) [ ] { } |
| `@property` | Property key names in map literals and SET/REMOVE |

### Decision

Use these capture names exactly as listed. Avoid custom non-standard names to ensure portability across editors.

---

## 3. Locals.scm Scope Modeling for Tree-sitter

### Finding

Tree-sitter's locals system uses three captures:
- `@local.scope`: Marks a scope boundary node (a subtree within which definitions are tracked)
- `@local.definition`: Marks a variable introduction (binding site)
- `@local.reference`: Marks a use of a previously defined name

For Cypher, scope semantics:
- A `source_file` or top-level `statement` is the outer scope
- `WITH` acts as a scope boundary — only projected variables remain in scope after it
- `CALL {}` subqueries introduce inner scopes
- `UNION` branches each have their own scope
- Variables introduced in `MATCH`/`CREATE` patterns are visible through the rest of the clause chain until the next `WITH` or end of statement

### Decision

Model scopes conservatively: mark the entire `statement` (and `union_statement`) as `@local.scope`. Within a statement, variables introduced in node/relationship pattern `variable` fields are `@local.definition`; uses of identifiers in expression positions are `@local.reference`. The `WITH` clause scope boundary is not explicitly modeled (full scope splitting requires editor support beyond basic locals).

This gives editors enough information for "rename symbol" and reference highlighting without incorrect cross-statement linking.

---

## 4. Tags.scm for Code Navigation

### Finding

Tree-sitter tags are used by tools like GitHub's code navigation and ctags-compatible indexers. The standard captures:
- `@definition.function` — function or procedure definitions/calls worth indexing
- `@name` — the name node within a definition (used to extract the symbol name)
- `@doc` — associated documentation (rarely used)

For Cypher, there are no "definitions" in the traditional sense (Cypher is a query language, not a definition language). However, `CALL` procedure references and user-defined function calls are the closest analog to navigable symbols.

### Decision

`tags.scm` emits `@definition.function` + `@name` for:
1. Procedure names in `call_clause` — the `procedure_name` node
2. Function names in `function_call` — the `function_name` node

This enables "go to references" for procedures across query files in workspace-aware editors.

---

## 5. Identifier Disambiguation in highlights.scm

### Finding

The current highlights.scm has a capture ordering issue: `(identifier) @variable` fires on ALL identifiers, including label names. The `label_expression (identifier) @type` capture also fires on the same identifiers. Editors typically use the **last** matching pattern, so `@variable` applied after `@type` would override the type highlight.

Tree-sitter applies captures in query order — later patterns override earlier ones for the same node. Ordering must be: specific patterns (labels, property keys, function names) BEFORE the generic `(identifier) @variable` fallback.

Additionally, property key names in `property_key_value` should be captured as `@property`, not `@variable`.

### Decision

Order highlights.scm patterns from most-specific to least-specific:
1. Keyword captures (new, after grammar change)
2. Comments
3. Literals (string, number, boolean, null)
4. Parameter
5. Labels and relationship types: `(label_expression … (identifier) @type)`
6. Function/procedure names: `(function_name (identifier) @function)`, `(procedure_name (identifier) @function)`
7. Property keys: `(property_key_value (identifier) @property)`
8. Operators and punctuation
9. Generic identifier fallback: `(identifier) @variable`, `(escaped_identifier) @variable`
