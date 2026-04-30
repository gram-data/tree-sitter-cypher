# Contract: Highlight Capture Interface

**Feature**: 002-syntax-highlighting | **Date**: 2026-04-29

This document defines the public interface exposed by the three query files. Editor plugins, nvim-treesitter configurations, and theme authors depend on these capture names remaining stable.

---

## highlights.scm Captures

The following capture groups are guaranteed to be produced. Editors may map these to theme variables; any name marked **stable** must not be renamed without a major version bump.

| Capture Name | Stability | Example Cypher Tokens |
|---|---|---|
| `@keyword` | stable | MATCH, RETURN, CREATE, MERGE, DELETE, SET, REMOVE, WITH, UNWIND, CALL, UNION, WHERE, ORDER, BY, SKIP, LIMIT, OFFSET, AS, ON, CASE, WHEN, THEN, ELSE, END, YIELD |
| `@keyword.operator` | stable | AND, OR, NOT, XOR, IN, IS, CONTAINS, STARTS, ENDS, ALL, ANY, NONE, SINGLE, REDUCE |
| `@keyword.control` | stable | OPTIONAL, DISTINCT, DETACH |
| `@keyword.modifier` | stable | ASC, DESC, ASCENDING, DESCENDING |
| `@string` | stable | `"hello"`, `'world'` |
| `@number` | stable | `42`, `3.14`, `0xFF`, path length `*1..5` |
| `@boolean` | stable | `true`, `false` |
| `@constant.builtin` | stable | `null` |
| `@variable` | stable | `n`, `m`, `person` (graph pattern variables) |
| `@variable.parameter` | stable | `$userId`, `$0` |
| `@type` | stable | `Person`, `KNOWS` (labels and relationship types) |
| `@function` | stable | `count`, `toUpper`, `apoc.load.json`, `COUNT(*)` |
| `@property` | stable | `name`, `age` (property key positions) |
| `@operator` | stable | `+`, `-`, `*`, `/`, `=`, `<>`, `->`, `<-`, etc. |
| `@comment` | stable | `// comment`, `/* block comment */` |
| `@punctuation.bracket` | stable | `(`, `)`, `[`, `]`, `{`, `}` |
| `@punctuation.delimiter` | stable | `,`, `;`, `.` |

---

## locals.scm Captures

| Capture Name | Stability | Semantics |
|---|---|---|
| `@local.scope` | stable | Scope boundary — `statement`, `union_statement` |
| `@local.definition` | stable | Variable introduction site |
| `@local.reference` | stable | Variable use site |

---

## tags.scm Captures

| Capture Name | Stability | Semantics |
|---|---|---|
| `@definition.function` | stable | Procedure or function name at a call site |
| `@name` | stable | The identifier subtree within a definition tag |

---

## Versioning Policy

- Capture names listed as **stable** are part of the public interface.
- Adding new captures is non-breaking.
- Renaming or removing stable captures requires updating this contract and bumping the grammar package version.
- Editors should fall back gracefully when a capture name is absent (no highlights, not an error).
