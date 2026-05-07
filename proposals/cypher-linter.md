## Cypher Linter: Query-Based Static Analysis

This document outlines the architecture for a Cypher linter built with Rust and Tree-sitter.
This approach moves beyond simple regex-based checks by using structural pattern matching
against the Abstract Syntax Tree (AST). With the addition of `tree-sitter-cypherdoc`, the
linter can also cross-reference declared intent (parameter names, types, return shape) against
the actual Cypher statement — catching mismatches that pure structural analysis cannot.

## Core Concept

Instead of hard-coding lint rules in Rust, we use Tree-sitter Query Files (`.scm`). These
files contain S-expressions that match specific shapes in the AST. By annotating these queries
with metadata, we create a declarative engine for code quality.

Two grammars work together:

- **`tree-sitter-cypher`** — parses the Cypher statement body (structural rules)
- **`tree-sitter-cypherdoc`** — parses the `/** */` doc comment (intent/contract rules)

Both trees are siblings under `source_file`. The linter loads both and correlates them
by position — the `doc_comment` node immediately preceding a `statement` node is its contract.

------------------------------

## 1. Rule Categories

### Structural rules (Cypher AST only)

These match patterns in the Cypher parse tree regardless of any doc comment.

```lisp
;; Rule: UnlabelledNode
;; Severity: Warning
;; Message: "MATCH (n)" causes a full node scan. Add a label, e.g., (n:Person).
(node_pattern
  variable: (identifier)
  !label)

;; Rule: UnboundedRelationship
;; Severity: Error
;; Message: Variable-length relationships without limits [r*] can hang the database.
(path_pattern
  (relationship_pattern
    (path_length) @unbounded)
  (#not-match? @unbounded "\\.\\."[0-9]))
```

### Contract rules (cypherdoc AST only)

These check the doc comment for internal consistency, independent of the Cypher body.

```lisp
;; Rule: OptionalParamMissingDefault
;; Severity: Error
;; Message: Optional @param must declare a default value — bare [name] is not allowed.
(document
  (param_tag
    param: (optional_param
      (MISSING))))

;; Rule: MissingToolName
;; Severity: Warning
;; Message: Cypherdoc comment has no tool name. Add a name as the first line.
(document
  name: (name
    (MISSING)))
```

### Cross-reference rules (both ASTs)

These correlate the cypherdoc contract against the Cypher statement. The linter loads both
trees for each file and runs cross-reference rules by pairing each `doc_comment` with the
`statement` that follows it (by source position).

```lisp
;; Rule: UndocumentedParameter
;; Severity: Warning
;; Message: "$X" is used in the query but not declared as @param in the doc comment.
;;
;; Pattern applied to the Cypher AST; the linter checks each captured $parameter
;; against the set of declared param names extracted from the cypherdoc AST.
(parameter) @used_param

;; Rule: UnusedParameter
;; Severity: Warning
;; Message: "@param X" is declared but $X never appears in the query.
;;
;; Pattern applied to the cypherdoc AST; the linter checks each declared name
;; against the set of $parameters found in the Cypher AST.
(param_tag
  param: [(required_param) (optional_param)]
    name: (identifier) @declared_param)

;; Rule: CardinalityMismatch
;; Severity: Warning
;; Message: "@returns" declares one row but query has no LIMIT 1 or unique key filter.
;;
;; Applied to Cypher AST when cypherdoc has @returns {[...]} (no array_marker).
(return_clause
  (return_body)) @return_clause
```

------------------------------

## 2. Cross-Tree Correlation

The linter pairs a `doc_comment` with its `statement` by adjacency in `source_file`:

```
source_file
  doc_comment   ← cypherdoc tree (injected)
  statement     ← cypher tree
  doc_comment   ← cypherdoc tree
  statement     ← cypher tree
```

In Rust, the linter walks `source_file` children, collecting `(doc_comment, statement)` pairs.
For each pair it runs:

1. Structural rules against the `statement` tree
2. Contract rules against the `doc_comment` / cypherdoc tree
3. Cross-reference rules using both trees together

For cross-reference rules, the linter extracts two sets and compares them:

```rust
let declared: HashSet<&str> = cypherdoc_query
    .matches(&cypherdoc_tree, source)
    .map(|m| m.captures["declared_param"].text())
    .collect();

let used: HashSet<&str> = cypher_query
    .matches(&cypher_tree, source)
    .map(|m| m.captures["used_param"].text().trim_start_matches('$'))
    .collect();

for unused in declared.difference(&used) {
    emit_warning(UnusedParameter, unused);
}
for undocumented in used.difference(&declared) {
    emit_warning(UndocumentedParameter, undocumented);
}
```

------------------------------

## 3. CLI Architecture (Rust)

The Rust binary handles file I/O, manages both Tree-sitter lifecycles, and formats output.

### The Lifecycle

1. **Loading**: Read `.scm` rule files and the `.cypher` source
2. **Parsing**: Run `tree-sitter-cypher` to get the Cypher tree; the injected
   `tree-sitter-cypherdoc` grammar parses each `doc_comment` node
3. **Pairing**: Walk `source_file` to build `(doc_comment, statement)` pairs
4. **Matching**: Run structural, contract, and cross-reference queries against each pair
5. **Reporting**: Emit diagnostics with line/column from the AST node ranges

### Recommended Crates

- `tree-sitter` — core parser and query engine
- `tree-sitter-cypher` — Cypher grammar (this repo)
- `tree-sitter-cypherdoc` — cypherdoc grammar (`tree-sitter-cypherdoc/` in this repo)
- `miette` or `ariadne` — compiler-grade error reporting with source snippets
- `clap` — CLI arguments (`--format json`, `--fix`, `--rule`)
- `rayon` — parallel processing of multiple `.cypher` files

------------------------------

## 4. Rule Definition Format

Rules are defined as `.scm` files with structured comment headers:

```lisp
;; Rule: UnusedParameter
;; Severity: Warning
;; Applies-to: cross-reference
;; Message: "@param {name}" is declared but ${name} never appears in the query.
(param_tag
  param: [(required_param) (optional_param)]
    name: (identifier) @declared_param)
```

The `Applies-to` header tells the engine which tree(s) to run the pattern against:
- `structural` — Cypher AST only
- `contract` — cypherdoc AST only
- `cross-reference` — both ASTs, post-correlation

------------------------------

## 5. Example Report Output

Using `miette`, the CLI output would look like this:

```
Warning: UnusedParameter
  ⚠ @param "label" is declared but $label never appears in the query.
   ╭─[find_person.cypher:6:4]
 6 │  * @param {string} [label="Person"] - Node label to filter by
   ·                    ──────
   ╰──── remove this @param or add $label to the MATCH clause

Warning: UndocumentedParameter
  ⚠ $limit is used in the query but not declared as @param in the doc comment.
   ╭─[get_colleagues.cypher:14:7]
14 │ LIMIT $limit
   ·       ──────
   ╰──── add @param {integer} [limit=25] - Maximum number of results to the doc comment
```

```
Error: UnboundedRelationship
  × Variable-length relationships without limits [r*] can hang the database.
   ╭─[shortest_path.cypher:11:38]
11 │   (a:Person {name: $from_name})-[*]-(b:Person {name: $to_name})
   ·                               ───
   ╰──── add a depth limit, e.g., [*..5]
```

------------------------------

## 6. Key Advantages

- **Structural rules** — Tree-sitter queries are highly optimised C-based searches
- **Contract rules** — cypherdoc gives the linter declared intent to check against
- **Cross-reference rules** — dead/undeclared parameters and cardinality mismatches
  are impossible to detect without both grammars working together
- **Portability** — single binary, no external database connection required
- **Extensibility** — adding a rule is adding lines to a `.scm` file; no Rust recompilation
  needed if rule files are loaded at runtime

------------------------------

## Appendix A — Future Directions: Auto-fixing

Because Tree-sitter nodes provide exact byte offsets (`node.start_byte()` and
`node.end_byte()`), the CLI can implement a `--fix` flag. For example:

- Add a missing `LIMIT` clause when `@returns` declares one row
- Scaffold a missing `@param` declaration for an undeclared `$parameter`
- Add `:Label` to an unlabelled node pattern

------------------------------

## Appendix B — Cypher Best Practices

See `references/neo4j-skills/` for examples of best practices that can inform structural
lint rules (unlabelled nodes, Cartesian products, missing index hints, etc.).
