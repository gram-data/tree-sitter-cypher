# tree-sitter-cypherdoc

[Tree-sitter](https://tree-sitter.github.io/tree-sitter/) grammar for structured `/** */` documentation comments in `.cypher` files.

This grammar is [injected](https://tree-sitter.github.io/tree-sitter/syntax-highlighting#language-injection) into Cypher `doc_comment` nodes by [`tree-sitter-cypher`](../) via `queries/injections.scm`.

## Comment Format

```cypher
/**
 * find_person_by_name
 *
 * Find a Person node by exact name match.
 *
 * @param {string} name - The full name to search for
 * @returns {[person: node<Person>]} - The matching person, or no rows if not found
 */
MATCH (person:Person {name: $name})
RETURN person
```

| Part | Required | Description |
|------|----------|-------------|
| name | Yes | Identifier on the first line — used as the query/tool name |
| description | No | Free-text lines below the name |
| `@param {type} name` | No | Required parameter |
| `@param {type} [name="default"]` | No | Optional parameter with a typed default value |
| `@returns {[col: type, ...]}` | No | Return columns as a named tuple |

## Install

**Node.js**
```sh
npm install tree-sitter-cypherdoc tree-sitter
```

**Rust**
```sh
cargo add tree-sitter-cypherdoc
```

## Usage

Normally consumed via the injection in `tree-sitter-cypher`. To parse cypherdoc comments directly:

**Node.js**
```js
import Parser from "tree-sitter";
import Cypherdoc from "tree-sitter-cypherdoc";

const parser = new Parser();
parser.setLanguage(Cypherdoc);

const tree = parser.parse(`/**
 * say_hello
 * @param {string} name - The name to greet
 * @returns {[greeting: string]}
 */`);
console.log(tree.rootNode.toString());
```

**Rust**
```rust
let mut parser = tree_sitter::Parser::new();
parser
    .set_language(&tree_sitter_cypherdoc::LANGUAGE.into())
    .expect("Error loading Cypherdoc parser");
```

## License

MIT
