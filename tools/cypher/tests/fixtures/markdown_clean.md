# Clean Cypher Example

This file contains a valid Cypher query inside a fenced code block.
It should produce no diagnostics.

```cypher
/**
 * find_person
 * @param {string} name - Person name
 * @returns {[{person: node}]} - Matching nodes
 */
MATCH (person:Person {name: $name})
RETURN person
```
