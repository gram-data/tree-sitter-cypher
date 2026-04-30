# Quickstart: Testing Syntax Highlighting

**Feature**: 002-syntax-highlighting | **Date**: 2026-04-29

---

## Prerequisites

```sh
npm install       # install tree-sitter CLI and bindings
tree-sitter generate   # regenerate parser after grammar.js changes
```

---

## Test Highlighting Output

Create a test file:

```sh
cat > /tmp/sample.cypher << 'EOF'
// Find all people who know Alice
MATCH (a:Person {name: $name})-[:KNOWS]->(b:Person)
WHERE b.age > 21
RETURN b.name AS friend, b.age
ORDER BY b.age DESC
LIMIT 10
EOF
```

Run the highlights query directly to see capture matches:

```sh
tree-sitter query queries/highlights.scm /tmp/sample.cypher
```

Each output line shows: pattern number, capture name, position, and matched text. Verify:
- Keywords (`MATCH`, `RETURN`, `WHERE`, etc.) appear as `@keyword`
- `Person`, `KNOWS` appear as `@type`
- `$name` appears as `@variable.parameter`
- `name`, `age` in property positions appear as `@property`
- `friend` (alias) does NOT appear as `@variable.parameter`
- `//` comment appears as `@comment`

---

## Test Locals (Variable Scoping)

```sh
tree-sitter query queries/locals.scm /tmp/sample.cypher
```

Verify:
- `a` and `b` in the MATCH pattern appear as `@local.definition`
- `b` in `WHERE b.age` and `RETURN b.name` appear as `@local.reference`
- `friend` (AS alias in RETURN) appears as `@local.definition`

---

## Test Tags

```sh
cat > /tmp/calls.cypher << 'EOF'
CALL apoc.load.json($url) YIELD value
RETURN toUpper(value.name) AS name
EOF

tree-sitter query queries/tags.scm /tmp/calls.cypher
```

Verify:
- `apoc.load.json` (procedure_name) appears with `@definition.function` + `@name`
- `toUpper` (function_name) appears with `@definition.function` + `@name`

---

## Visual Highlight Test (requires editor setup)

In Neovim with nvim-treesitter pointing to this grammar:

```
:TSHighlightCapturesUnderCursor
```

Or run the tree-sitter CLI playground:

```sh
npm start
```

---

## Run Grammar Tests

After grammar.js changes for keyword aliasing:

```sh
tree-sitter generate   # must run first
tree-sitter test       # all corpus tests must pass
make test              # alias for tree-sitter test
```

The keyword aliasing change should not break any existing corpus tests — it changes only how keyword tokens appear in the AST (as aliased anonymous nodes instead of invisible tokens).
