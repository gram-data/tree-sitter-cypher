# tree-sitter-cypher

[Tree-sitter](https://tree-sitter.github.io/tree-sitter/) grammar for [Cypher](https://opencypher.org/), the graph query language used by Neo4j and other graph databases.

## Install

**Node.js**
```sh
npm install tree-sitter-cypher tree-sitter
```

**Rust**
```sh
cargo add tree-sitter-cypher
```

## Usage

**Node.js**
```js
import Parser from "tree-sitter";
import Cypher from "tree-sitter-cypher";

const parser = new Parser();
parser.setLanguage(Cypher);

const tree = parser.parse("MATCH (n:Person {name: $name}) RETURN n");
console.log(tree.rootNode.toString());
```

**Rust**
```rust
let mut parser = tree_sitter::Parser::new();
parser
    .set_language(&tree_sitter_cypher::LANGUAGE.into())
    .expect("Error loading Cypher parser");

let tree = parser.parse("MATCH (n:Person {name: $name}) RETURN n", None).unwrap();
assert!(!tree.root_node().has_error());
```

## Queries

The `queries/` directory contains Tree-sitter query files for editor integration:

| File | Purpose |
|------|---------|
| `highlights.scm` | Syntax highlighting |
| `injections.scm` | Injects `tree-sitter-cypherdoc` into `/** */` doc comments |
| `tags.scm` | Symbol tagging for code navigation |
| `locals.scm` | Variable scope tracking |

## Related Packages

- **[tree-sitter-cypherdoc](./tree-sitter-cypherdoc)** — Grammar for structured `/** */` doc comments in `.cypher` files
- **[cypher](./tools/cypher)** — CLI linter for `.cypher` files

## License

MIT
