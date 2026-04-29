<!--
SYNC IMPACT REPORT
==================
Version change: (template) → 1.0.0
Added principles:
  - I. Spec Fidelity (new)
  - II. Dual-Coverage Testing (new)
  - III. TCK Validation (new)
Removed sections: all placeholder template sections
Templates reviewed:
  - .specify/templates/plan-template.md       ✅ Constitution Check section aligns
  - .specify/templates/spec-template.md       ✅ No structural changes required
  - .specify/templates/tasks-template.md      ✅ Test-first pattern aligns with Principle II
Deferred TODOs: none
-->

# tree-sitter-cypher Constitution

## Core Principles

### I. Spec Fidelity

Every grammar rule in `grammar.js` MUST correspond to a named production in the openCypher BNF
(`references/openCypher/grammar/openCypher.bnf`). Rules that deviate from or extend the BNF
MUST be explicitly documented with the rationale, and MUST NOT silently accept input the spec
rejects.

- Grammar rule names MUST be derived from the BNF production names (snake_case translation).
- No rule may be added purely for implementation convenience without a BNF anchor.
- When the BNF is ambiguous, the TCK is the authoritative tiebreaker.

### II. Dual-Coverage Testing

Every grammar rule MUST be covered by at least one **positive** corpus test (valid input that
parses without ERROR nodes) and at least one **negative** corpus test (invalid input that
produces an ERROR node in the expected location).

- Corpus tests live in `test/corpus/`, organized by the 8 implementation slices (P1–P8).
- A rule with only positive tests is considered incomplete.
- A rule with only negative tests is considered incomplete.
- `tree-sitter test` MUST pass with zero failures before any slice is considered done.

### III. TCK Validation

The grammar MUST be validated against the openCypher Technology Compatibility Kit (TCK) at
`references/openCypher/tck/`. Parsing any Cypher query from a TCK feature file MUST produce
a tree with zero ERROR nodes.

- TCK validation is the final acceptance gate for each implementation slice.
- A slice is not complete until all TCK queries relevant to its feature area parse cleanly.
- TCK failures block merge; they are not deferred to a later slice.

## Quality Gates

The following gates MUST pass before any implementation slice is considered shippable:

| Gate | Check |
|------|-------|
| **Fidelity gate** | Every new rule traces to a BNF production (document in code comment or PR) |
| **Dual-coverage gate** | `tree-sitter test` reports ≥1 positive AND ≥1 negative test per rule |
| **TCK gate** | `tree-sitter parse` on all relevant TCK `.cypher` snippets reports zero ERROR nodes |

## Governance

- This constitution is amended by updating this file with a version bump and a rationale entry.
- **PATCH** (x.y.Z): Clarifications or wording improvements to existing principles.
- **MINOR** (x.Y.z): New principle or quality gate added.
- **MAJOR** (X.y.z): Existing principle removed or fundamentally redefined.
- All pull requests touching `grammar.js` or `test/corpus/` MUST reference which constitution
  gate(s) they satisfy in the PR description.
- The constitution supersedes any other development guidance when they conflict.

**Version**: 1.0.0 | **Ratified**: 2026-04-29 | **Last Amended**: 2026-04-29
