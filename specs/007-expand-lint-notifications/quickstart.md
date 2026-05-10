# Quickstart: Expanded Lint Notifications

After implementation, these new warnings are available out of the box.

## Cartesian Product — `03N90`

```cypher
# Bad: disconnected MATCH produces a cross-product
MATCH (u:User), (o:Order) RETURN u, o
# [CartesianProduct/03N90] Warning: Multiple comma-separated MATCH patterns may produce a cartesian product...

# Good: connected patterns
MATCH (u:User)-[:PLACED]->(o:Order) RETURN u, o
```

## Deprecated `id()` — `01N01`

```cypher
# Bad: id() is deprecated in Neo4j 5
MATCH (n:Person) RETURN id(n)
# [DeprecatedFunction/01N01] Warning: id() is deprecated in Neo4j 5...

# Good: use elementId()
MATCH (n:Person) RETURN elementId(n)
```

## Dynamic Property Access — `03N95`

```cypher
# Bad: prevents index use
MATCH (n:Person) WHERE n[$prop] IS NOT NULL RETURN n
# [DynamicProperty/03N95] Advice: Dynamic property key prevents index use...

# Good: static property name
MATCH (n:Person) WHERE n.name IS NOT NULL RETURN n
```

## JSON Output with Codes

```sh
cypher lint --json query.cypher
```

```json
{
  "schema_version": 1,
  "tool": "cypher/0.2.3",
  "files": [
    {
      "path": "query.cypher",
      "diagnostics": [
        {
          "severity": "warning",
          "rule": "CartesianProduct",
          "message": "Disconnected MATCH patterns produce a cartesian product...",
          "range": { "start": {"line": 0, "character": 16}, "end": {"line": 0, "character": 25} },
          "code": "03N90"
        }
      ]
    }
  ]
}
```

The `code` field maps directly to the [Neo4j notification reference](https://neo4j.com/docs/status-codes/current/notifications/all-notifications/).
