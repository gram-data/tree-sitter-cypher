# Contract: JSON Output Schema

**Feature**: 005-cypher-cli | **Date**: 2026-05-08  
**Triggered by**: `cypher lint --json`

## Schema Version

`schema_version: 1` — increment the minor version when fields are added; increment the major
version when fields are removed or renamed.

## Top-Level Object

```json
{
  "schema_version": 1,
  "tool": "cypher/0.1.0",
  "files": [ <FileResult>, ... ]
}
```

| Field | Type | Always present | Notes |
|-------|------|----------------|-------|
| `schema_version` | `integer` | yes | Always `1` in this version |
| `tool` | `string` | yes | `"cypher/<semver>"` |
| `files` | `array` | yes | One entry per checked file; empty array if no files processed |

## FileResult Object

```json
{
  "path": "src/queries/find_person.cypher",
  "diagnostics": [ <Diagnostic>, ... ]
}
```

| Field | Type | Always present | Notes |
|-------|------|----------------|-------|
| `path` | `string` | yes | Relative or absolute path as given; `-` for stdin |
| `diagnostics` | `array` | yes | Empty array when file is clean |

## Diagnostic Object

```json
{
  "severity": "warning",
  "rule": "UnlabelledNode",
  "message": "MATCH (n) causes a full node scan. Add a label, e.g., (n:Person).",
  "range": {
    "start": { "line": 4, "character": 7 },
    "end":   { "line": 4, "character": 10 }
  }
}
```

| Field | Type | Always present | Notes |
|-------|------|----------------|-------|
| `severity` | `string` | yes | One of `"error"`, `"warning"`, `"information"`, `"hint"` |
| `rule` | `string` | yes | PascalCase rule name, e.g., `"UnlabelledNode"` |
| `message` | `string` | yes | Human-readable description of the finding |
| `range` | `object` | yes | Source location (see below) |
| `code` | `string` | no | Optional machine-readable rule code; omitted when absent |

## Range Object

Line and character are **zero-indexed**, UTF-16 code units — matching the LSP specification
and consistent with gram's JSON output.

```json
{
  "start": { "line": 0, "character": 0 },
  "end":   { "line": 0, "character": 5 }
}
```

## Full Example

```json
{
  "schema_version": 1,
  "tool": "cypher/0.1.0",
  "files": [
    {
      "path": "queries/find_person.cypher",
      "diagnostics": [
        {
          "severity": "warning",
          "rule": "UnlabelledNode",
          "message": "MATCH (n) causes a full node scan. Add a label, e.g., (n:Person).",
          "range": {
            "start": { "line": 2, "character": 7 },
            "end":   { "line": 2, "character": 10 }
          }
        },
        {
          "severity": "warning",
          "rule": "UnusedParameter",
          "message": "@param \"label\" is declared but $label never appears in the query.",
          "range": {
            "start": { "line": 0, "character": 12 },
            "end":   { "line": 0, "character": 17 }
          }
        }
      ]
    },
    {
      "path": "queries/clean.cypher",
      "diagnostics": []
    }
  ]
}
```

## Compatibility with gram JSON Output

The schema mirrors `gram check --json` (`gram/src/types.rs`): same field names, same severity
strings, same range convention. The only addition is the `rule` field on `Diagnostic`. Tools
that consume `gram` JSON output can consume `cypher lint` JSON output with minimal changes.
