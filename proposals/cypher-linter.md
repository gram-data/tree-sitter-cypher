## Cypher Linter: Query-Based Static Analysis

This document outlines the architecture for a Cypher linter built with Rust and Tree-sitter. This approach moves beyond simple regex-based checks by using structural pattern matching against the Abstract Syntax Tree (AST).

## Core Concept

Instead of hard-coding lint rules in Rust, we use Tree-sitter Query Files (.scm). These files contain S-expressions that match specific shapes in the Cypher AST. By annotating these queries with metadata, we create a declarative engine for code quality.

------------------------------

## 1. Rule Definition (lint.scm)
Rules are defined as patterns. Each pattern uses captures (prefixed with @) to identify the offending code and comments to provide context for the CLI.

```lisp
;; Rule: UnlabelledNode;; Severity: Warning;; Message: "MATCH (n)" causes a full node scan. Add a label, e.g., (n:Person).
(node_pattern 
  (variable) 
  (node_labels)? @missing_labels
  (#is-not? @missing_labels))
;; Rule: UnboundedRelationship;; Severity: Error;; Message: Variable-length relationships without limits [r*] can hang the database.
(link_dash) @unbounded_rel
```

------------------------------
## 2. CLI Architecture (Rust)

The Rust binary acts as the runner. It handles file I/O, manages the Tree-sitter lifecycle, and formats the output.

## The Lifecycle

   1. Loading: The CLI reads the lint.scm file and the Cypher source code.
   2. Parsing: Tree-sitter converts the Cypher string into a Tree.
   3. Matching: The QueryCursor executes all patterns in lint.scm against the tree in a single pass.
   4. Reporting: For every match, the tool retrieves the line/column data from the AST node.

## Recommended Crates

* tree-sitter: The core parser and query engine.
* miette or ariadne: For "compiler-grade" error reporting with snippets and colors.
* clap: For handling CLI arguments (e.g., --format json, --fix).
* rayon: To process multiple .cypher files in parallel across CPU cores.

------------------------------

## 3. Implementation Workflow## Metadata Extraction

Since Tree-sitter's Query API doesn't natively parse "metadata" from comments, the Rust tool should:

* Read the .scm file as a string.
* Parse the custom headers (;; Rule:, ;; Message:) using a simple regex or line-iterator.
* Map the pattern_index returned by the query engine back to these metadata objects.

## Example Report Output

Using a crate like miette, the CLI output would look like this:

```bash
Error: UnlabelledNode
  × MATCH (n) causes a full node scan.
   ╭─[query.cypher:1:7]
 1 │ MATCH (n) RETURN n
   ·       ─┬─
   ·        ╰── Add a label, e.g., (n:Person).
   ╰────
```

------------------------------

## 4. Key Advantages

* Performance: Tree-sitter queries are highly optimised C-based searches.
* Portability: Compiling the grammar via a build.rs script produces a single binary that requires no external dependencies or database connection.
* Extensibility: Adding a new rule is as simple as adding a few lines to the .scm file; no Rust recompilation is strictly necessary if the file is loaded at runtime.

------------------------------

## Appendix A -- Future Directions: Auto-fixing

Because Tree-sitter nodes provide exact byte offsets (node.start_byte() and node.end_byte()), the CLI can implement an --fix flag. The tool can calculate the necessary string replacement (e.g., adding :Label to a node) and write the updated buffer back to the file.

------------------------------

## Appendix B -- Cypher Best Practices

See the git submodule at `references/neo4j-skills/` for many examples of best practices
which can inform lint rules.
