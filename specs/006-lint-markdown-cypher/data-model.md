# Data Model: Lint Markdown Cypher Snippets

**Feature**: 006-lint-markdown-cypher | **Date**: 2026-05-10

## New Type: `CypherSnippet`

Lives in `tools/cypher/src/markdown.rs`.

| Field | Type | Description |
|-------|------|-------------|
| `content` | `String` | The raw Cypher text inside the fence (excluding the fence markers themselves) |
| `start_line` | `u32` | Zero-based index of the first content line in the source markdown file |

**Invariant**: `start_line` is the line immediately following the opening ` ``` ` fence line,
not the fence line itself. This is the line number to add to snippet-relative diagnostic line
numbers to get markdown-absolute line numbers.

## Extended: `SourceResult` (internal, `lint.rs`)

No field additions. The existing fields are reused with different semantics for markdown files:

| Field | Existing Use | Markdown Use |
|-------|-------------|--------------|
| `path` | `.cypher` file path | `.md` file path |
| `source` | `.cypher` file content | Full `.md` file content (required for ariadne rendering with absolute line numbers) |
| `diags` | Diagnostics from `.cypher` parse | All diagnostics from all snippets, with line numbers offset to be absolute in the `.md` file |

## Extended: `LintArgs` (clap struct, `lint.rs`)

New field added to `LintArgs`:

| Field | Clap flag | Type | Description |
|-------|-----------|------|-------------|
| `no_markdown` | `--no-markdown` | `bool` | Skip `.md` files during directory traversal and explicit-path processing |

## Unchanged Types

`Diagnostic`, `FileResult`, `LintResult`, `Position`, `Range`, `Severity` — all unchanged.
The JSON output schema (`schema_version: 1`) is not bumped; the new fields are not required and
the existing fields are sufficient to represent markdown-file diagnostics.

## Extraction Algorithm (pseudo-code)

```text
fn extract_cypher_snippets(source) -> Vec<CypherSnippet>:
    snippets = []
    in_fence = false
    fence_start_line = 0
    content_lines = []

    for (line_num, line) in enumerate(source.lines()):
        if not in_fence:
            if is_cypher_fence_open(line):
                in_fence = true
                fence_start_line = line_num + 1   # content starts next line
                content_lines = []
        else:
            if is_fence_close(line):
                snippets.push(CypherSnippet {
                    content: content_lines.join("\n"),
                    start_line: fence_start_line
                })
                in_fence = false
            else:
                content_lines.push(line)

    # Unclosed fence: push remaining as a snippet (warn at lint level)
    if in_fence and not content_lines.is_empty():
        snippets.push(CypherSnippet {
            content: content_lines.join("\n"),
            start_line: fence_start_line
        })

    return snippets

fn is_cypher_fence_open(line) -> bool:
    trimmed = line.trim_start()
    rest = trimmed.strip_prefix("```") else return false
    lang = rest.split_whitespace().next().unwrap_or("").to_lowercase()
    return lang == "cypher"

fn is_fence_close(line) -> bool:
    return line.trim() == "```"
```

## Diagnostic Offset Application

After `analyze(snippet.content, md_path, rules)` returns a `SourceResult`:

```text
for diag in &mut source_result.diags:
    diag.range.start.line += snippet.start_line
    diag.range.end.line   += snippet.start_line
```

Then set `source_result.source = full_markdown_text` so ariadne renders with correct context.
