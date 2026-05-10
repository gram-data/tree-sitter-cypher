# Multiple Snippet Example

This file has three Cypher code blocks at different positions.

## Clean Query

```cypher
MATCH (p:Person) RETURN p
```

## Unlabelled Node

The following query triggers UnlabelledNode:

```cypher
MATCH (n)
RETURN n
```

## Unbounded Relationship

The following query triggers UnboundedRelationship:

```cypher
MATCH (a)-[*]->(b) RETURN a, b
```
