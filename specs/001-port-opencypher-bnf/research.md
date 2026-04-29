# Research: Port openCypher BNF to Tree-sitter Grammar

## Decision 1: Case-insensitive Keywords

**Decision**: Implement a `kw(str)` helper function in `grammar.js` that builds a case-insensitive regex for each keyword character.

```js
const kw = str =>
  token(new RegExp(
    str.split('').map(c =>
      /[a-zA-Z]/.test(c) ? `[${c.toUpperCase()}${c.toLowerCase()}]` : c
    ).join('')
  ));
```

Usage: `kw('MATCH')` → `/[Mm][Aa][Tt][Cc][Hh]/`

**Rationale**: Tree-sitter has no built-in case-insensitivity flag. The `token()` wrapper marks the rule as a terminal (leaf) so tree-sitter does not create a named node for the keyword itself — it appears inline as an anonymous token, keeping the AST clean. A helper avoids repeating the regex construction for every keyword.

**Alternatives considered**:
- `choice('MATCH', 'match', 'Match', ...)` — exhaustive, combinatorially impractical
- Lowercasing input before parse — transforms source positions, breaks editor integrations
- `token.immediate(...)` — unnecessary here; only needed to prevent whitespace between tokens

---

## Decision 2: BNF Construct → Tree-sitter DSL Mapping

**Decision**: Map openCypher BNF constructs to tree-sitter DSL functions as follows:

| BNF notation | Tree-sitter DSL |
|---|---|
| `A B C` (sequence) | `seq(A, B, C)` |
| `A \| B \| C` (alternation) | `choice(A, B, C)` |
| `[ A ]` (optional) | `optional(A)` |
| `{ A }...` (one or more) | `repeat1(A)` |
| `[ { A }... ]` (zero or more) | `repeat(A)` |
| `A [ { , A }... ]` (comma-list) | custom `commaSep1(A)` helper |

```js
const commaSep1 = rule => seq(rule, repeat(seq(',', rule)));
const commaSep  = rule => optional(commaSep1(rule));
```

**Rationale**: Tree-sitter's DSL maps 1:1 to these BNF patterns. The `commaSep` helpers avoid repeating the `seq(rule, repeat(seq(',', rule)))` pattern for every list production (return items, set items, procedure args, etc.).

---

## Decision 3: Operator Precedence

**Decision**: Express Cypher expression precedence using `prec.left` and `prec.right` with numeric levels matching the BNF nesting depth:

| Level | Operator(s) | Associativity |
|---|---|---|
| 1 | `OR` | left |
| 2 | `XOR` | left |
| 3 | `AND` | left |
| 4 | `NOT` | right (unary prefix) |
| 5 | `=`, `<>`, `<`, `>`, `<=`, `>=`, `=~` | left (comparison) |
| 6 | `+`, `-` (binary) | left |
| 7 | `*`, `/`, `%` | left |
| 8 | `^` (exponent) | right |
| 9 | `+`, `-` (unary) | right |
| 10 | `.`, `[]`, `()` (postfix) | left |

**Rationale**: The BNF defines precedence structurally through rule nesting (`boolean_value_expression` > `boolean_term` > `boolean_factor` > …). Tree-sitter's flat `prec` system encodes the same order without requiring separate named rules for every level, keeping `grammar.js` readable.

**Alternatives considered**: Preserving the full BNF nesting structure (one rule per precedence level) — syntactically faithful but creates ~10 intermediate node types that are noise in the AST for consumers.

---

## Decision 4: Comments and Whitespace as Extras

**Decision**: Register line comments and block comments in the `extras` array of the grammar so they are silently skipped between any two tokens:

```js
extras: $ => [
  /\s+/,
  /\/\/.*/,             // line comment
  /\/\*[^*]*\*+([^/*][^*]*\*+)*\//, // block comment
],
```

**Rationale**: Tree-sitter `extras` are interleaved anywhere in the parse stream. This matches Cypher's rule that comments and whitespace can appear between any two tokens. No named nodes are produced for comments, matching the spec requirement that they are skipped.

---

## Decision 5: Grammar Authoring Order (Bottom-up)

**Decision**: Implement rules in dependency order — terminal rules first, then composite rules — matching the 8 spec story slices:

1. Terminals (literals, identifiers, operators, keywords) — P1
2. Top-level statement + MATCH/RETURN clause frame + minimal node pattern — P2
3. Full graph pattern rules (relationship patterns, path variables, label expressions) — P3
4. Expression rules with precedence — P4
5. Mutation clauses (CREATE, SET, REMOVE, DELETE) — P5
6. Pipeline clauses (WITH, UNWIND, ORDER BY, SKIP, LIMIT) — P6
7. MERGE and CALL/YIELD — P7
8. UNION + advanced expressions (CASE, list comprehension, reduce, quantifiers) — P8

**Rationale**: Tree-sitter allows forward-referenced rules (all rules are resolved at grammar generation time), so authoring order doesn't affect correctness. However, writing terminals first means corpus tests for P1 can pass before any clause-level rules exist, providing an early pass/fail signal.

---

## Decision 6: Named vs. Anonymous Nodes

**Decision**: Make all semantically meaningful constructs named rules (appear in `node-types.json`); keep syntactic punctuation and keywords as anonymous inline tokens.

Examples:
- `match_clause`, `node_pattern`, `relationship_pattern`, `property_access` → named
- `(`, `)`, `-`, `->`, `MATCH`, `WHERE` → anonymous (string literals or `kw()` calls inline)

**Rationale**: Named nodes form the public contract for all tree-sitter consumers (editors, linters, analyzers). Punctuation and keywords are structural noise that consumers typically skip. The spec assumption "use named nodes for all meaningful syntactic elements" is enforced here.
