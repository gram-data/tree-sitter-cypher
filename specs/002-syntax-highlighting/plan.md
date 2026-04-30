# Implementation Plan: Cypher Syntax Highlighting and Code Navigation

**Branch**: `002-syntax-highlighting` | **Date**: 2026-04-29 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/002-syntax-highlighting/spec.md`

## Summary

Implement `highlights.scm`, `locals.scm`, and `tags.scm` query files to enable syntax highlighting and code navigation for Cypher in tree-sitter-enabled editors. The grammar currently uses regex-based anonymous keyword tokens that produce no AST nodes, so a prerequisite grammar change is needed to expose keywords as capturable named nodes before the query files can provide full coverage.

## Technical Context

**Language/Version**: Tree-sitter query language (Scheme-like s-expressions); Grammar source in JavaScript (ESM, Node.js)  
**Primary Dependencies**: tree-sitter CLI, generated `src/node-types.json` (authoritative node type reference)  
**Storage**: N/A — three files in `queries/`: `highlights.scm`, `locals.scm`, `tags.scm`  
**Testing**: `tree-sitter query <query-file> <example-file>`, `tree-sitter highlight <file>`, `make test`  
**Target Platform**: Tree-sitter editors: Neovim (nvim-treesitter), Helix, Zed; tree-sitter CLI tools  
**Project Type**: Grammar + query library  
**Performance Goals**: No explicit performance constraint; query execution is handled by the tree-sitter engine  
**Constraints**: All node type references in .scm files must exist in `src/node-types.json`; all capture names must follow the standard tree-sitter highlight taxonomy  
**Scale/Scope**: ~65 named node types; ~30 anonymous punctuation tokens; highlights.scm ~60-80 patterns, locals.scm ~15 patterns, tags.scm ~5 patterns

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate | Status | Notes |
|------|--------|-------|
| **Fidelity gate** | ✅ Applicable | Grammar changes for keyword exposure map to BNF keyword terminals |
| **Dual-coverage gate** | ✅ Applicable | Any grammar rule change requires positive + negative corpus tests |
| **TCK gate** | ✅ Applicable | Grammar changes must not regress existing TCK query parsing |
| **Query-only changes** | ✅ N/A for .scm files | The three query files themselves do not add grammar rules; constitution gates apply only to any grammar.js changes |

**Note**: This feature primarily delivers `.scm` query files, which are not subject to the constitution's grammar gates. However, a prerequisite grammar change (keyword node exposure) is required and IS subject to all three gates.

## Critical Finding: Keywords Are Not AST Nodes

**Investigation result**: The current grammar implements keywords via `kw()` which creates `token(new RegExp(...))` — regex-based anonymous tokens. These tokens are consumed by the parser but produce **zero nodes in the AST**. Parsing `MATCH (n:Person) RETURN n` yields no nodes for `MATCH` or `RETURN`; only `(`, `:`, `)` (string literal tokens) and named nodes like `identifier` appear.

**Impact**: `highlights.scm` cannot capture keywords at all without a grammar change.

**Required grammar change**: Convert keyword terminals to either (a) named rules (`kw_match`, `kw_return`, …) that produce capturable nodes, or (b) use `alias()` to assign a keyword supertype to each keyword occurrence. See `research.md` for the decision and rationale.

## Project Structure

### Documentation (this feature)

```text
specs/002-syntax-highlighting/
├── plan.md              ← this file
├── research.md          ← Phase 0 output
├── data-model.md        ← Phase 1 output (capture taxonomy)
├── contracts/
│   └── highlight-captures.md   ← Phase 1 output
└── tasks.md             ← Phase 2 output (/speckit-tasks command)
```

### Source Files (this feature)

```text
grammar.js               ← prerequisite grammar change (keyword nodes)
queries/
├── highlights.scm       ← primary deliverable
├── locals.scm           ← primary deliverable
├── tags.scm             ← primary deliverable
└── injections.scm       ← OUT OF SCOPE (already exists)

test/corpus/
└── highlights.txt       ← new corpus tests for keyword + capture correctness
```

## Complexity Tracking

No constitution violations requiring justification. The grammar change is additive (exposes existing tokens as named nodes) and maps directly to BNF keyword terminals.
