# Unlabelled Node Example

This file demonstrates the UnlabelledNode lint rule embedded in markdown.

The following Cypher query has an unlabelled node pattern and should trigger
a warning for the `(n)` node with no label.

```cypher
MATCH (n)
RETURN n
```
