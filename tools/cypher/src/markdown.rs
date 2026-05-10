#[derive(Debug, Clone, PartialEq)]
pub struct CypherSnippet {
    pub content: String,
    /// Zero-based index of the first content line in the source markdown file.
    pub start_line: u32,
    /// `false` when end-of-file was reached before the closing ` ``` ` fence.
    pub closed: bool,
}

pub fn extract_cypher_snippets(source: &str) -> Vec<CypherSnippet> {
    let mut snippets = Vec::new();
    let mut in_fence = false;
    let mut fence_start_line: u32 = 0;
    let mut content_lines: Vec<&str> = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if !in_fence {
            if is_cypher_fence_open(line) {
                in_fence = true;
                fence_start_line = (line_num + 1) as u32;
                content_lines.clear();
            }
        } else if is_fence_close(line) {
            snippets.push(CypherSnippet {
                content: content_lines.join("\n"),
                start_line: fence_start_line,
                closed: true,
            });
            in_fence = false;
        } else {
            content_lines.push(line);
        }
    }
    // Unclosed fence: include remaining content, mark as unclosed
    if in_fence && !content_lines.is_empty() {
        snippets.push(CypherSnippet {
            content: content_lines.join("\n"),
            start_line: fence_start_line,
            closed: false,
        });
    }
    snippets
}

fn is_cypher_fence_open(line: &str) -> bool {
    let trimmed = line.trim_start();
    let rest = match trimmed.strip_prefix("```") {
        Some(r) => r,
        None => return false,
    };
    let lang = rest.split_whitespace().next().unwrap_or("");
    lang.eq_ignore_ascii_case("cypher")
}

fn is_fence_close(line: &str) -> bool {
    line.trim() == "```"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_snippet() {
        let src = "# Title\n\n```cypher\nMATCH (n) RETURN n\n```\n";
        let snippets = extract_cypher_snippets(src);
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].content, "MATCH (n) RETURN n");
        assert_eq!(snippets[0].start_line, 3);
        assert!(snippets[0].closed);
    }

    #[test]
    fn multiple_snippets() {
        let src = "```cypher\nA\n```\n\n```cypher\nB\n```\n";
        let snippets = extract_cypher_snippets(src);
        assert_eq!(snippets.len(), 2);
        assert_eq!(snippets[0].content, "A");
        assert_eq!(snippets[0].start_line, 1);
        assert!(snippets[0].closed);
        assert_eq!(snippets[1].content, "B");
        assert_eq!(snippets[1].start_line, 5);
        assert!(snippets[1].closed);
    }

    #[test]
    fn no_snippets() {
        assert!(extract_cypher_snippets("# Just prose\nNo code here.\n").is_empty());
    }

    #[test]
    fn empty_snippet_returned() {
        let src = "```cypher\n```\n";
        let snippets = extract_cypher_snippets(src);
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].content, "");
        assert_eq!(snippets[0].start_line, 1);
        assert!(snippets[0].closed);
    }

    #[test]
    fn mixed_case_fence() {
        let src = "```Cypher\nMATCH (n) RETURN n\n```\n";
        assert_eq!(extract_cypher_snippets(src).len(), 1);
    }

    #[test]
    fn fence_with_trailing_annotation() {
        let src = "```cypher title=\"example\"\nMATCH (n) RETURN n\n```\n";
        assert_eq!(extract_cypher_snippets(src).len(), 1);
    }

    #[test]
    fn other_language_tags_skipped() {
        let src = "```cql\nSELECT *\n```\n```cypher-shell\n:play\n```\n```cypherdoc\nfoo\n```\n";
        assert!(extract_cypher_snippets(src).is_empty());
    }

    #[test]
    fn unclosed_fence_marked_not_closed() {
        let src = "```cypher\nMATCH (n) RETURN n\n";
        let snippets = extract_cypher_snippets(src);
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].content, "MATCH (n) RETURN n");
        assert_eq!(snippets[0].start_line, 1);
        assert!(!snippets[0].closed);
    }

    #[test]
    fn start_line_offset_is_correct() {
        // Fence opens at line 6 (0-based), content starts at line 7 (0-based) = start_line 7
        let src = "line0\nline1\nline2\nline3\nline4\nline5\n```cypher\nMATCH (n) RETURN n\n```\n";
        let snippets = extract_cypher_snippets(src);
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].start_line, 7);
        assert!(snippets[0].closed);
    }
}
