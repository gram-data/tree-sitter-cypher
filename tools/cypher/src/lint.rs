use ariadne::{Color, Label, Report, ReportKind, sources};
use clap::Args;
use std::collections::HashSet;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use tree_sitter::{Node, StreamingIterator, Tree};
use walkdir::WalkDir;

use crate::markdown::extract_cypher_snippets;
use crate::rules::{AppliesTo, Rule, builtin_rules, parse_rule_file};
use crate::types::{Diagnostic, FileResult, LintResult, Position, Range, Severity};

#[derive(Args)]
#[command(about = "Lint .cypher files for parse and semantic errors")]
pub struct LintArgs {
    /// Lint an inline Cypher expression
    #[arg(short = 'e', long = "expression")]
    pub expression: Option<String>,

    /// Output diagnostics as machine-readable JSON
    #[arg(long, conflicts_with = "tree")]
    pub json: bool,

    /// Output the parse tree as an s-expression
    #[arg(long, conflicts_with = "json")]
    pub tree: bool,

    /// Treat warnings as errors (exit non-zero on any diagnostic)
    #[arg(long)]
    pub strict: bool,

    /// Run only the named rule(s) (repeatable)
    #[arg(long = "rule")]
    pub rule: Vec<String>,

    /// Load additional .scm rule files from this directory
    #[arg(long = "rules-dir")]
    pub rules_dir: Option<PathBuf>,

    /// Skip .md files during directory traversal and explicit-path processing
    #[arg(long = "no-markdown")]
    pub no_markdown: bool,

    /// Files, directories, or paths to lint (omit to read from stdin)
    #[arg(num_args = 0..)]
    pub paths: Vec<PathBuf>,
}

pub struct LintOptions {
    pub strict: bool,
}

pub fn lint_source(source: &str, _options: &LintOptions) -> Vec<Diagnostic> {
    let rules = builtin_rules();
    analyze(source.to_owned(), "-".to_owned(), &rules).diags
}

pub fn lint_file(path: &Path, options: &LintOptions) -> anyhow::Result<Vec<Diagnostic>> {
    let source = std::fs::read_to_string(path)?;
    Ok(lint_source(&source, options))
}

struct SourceResult {
    path: String,
    source: String,
    diags: Vec<Diagnostic>,
}

struct DocStatementPair {
    doc_start_byte: Option<usize>,
    doc_end_byte: Option<usize>,
    statement_start_byte: usize,
    statement_end_byte: usize,
}

pub fn run(args: LintArgs) -> i32 {
    if args.tree {
        return run_tree(&args);
    }

    let mut rules = builtin_rules();

    if let Some(dir) = &args.rules_dir {
        let cypher_lang: tree_sitter::Language = tree_sitter_cypher::LANGUAGE.into();
        let cypherdoc_lang: tree_sitter::Language = tree_sitter_cypherdoc::LANGUAGE.into();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("scm") {
                    match std::fs::read_to_string(&path) {
                        Ok(src) => {
                            // Pick language from the Applies-to: header so contract rules work.
                            let lang = if applies_to_header(&src) == "contract" {
                                cypherdoc_lang.clone()
                            } else {
                                cypher_lang.clone()
                            };
                            match parse_rule_file(&src, lang) {
                                Ok(r) => rules.push(r),
                                Err(e) => eprintln!("warning: {}: {e}", path.display()),
                            }
                        }
                        Err(e) => eprintln!("warning: {}: {e}", path.display()),
                    }
                }
            }
        }
    }

    if !args.rule.is_empty() {
        let unknown: Vec<&String> = args
            .rule
            .iter()
            .filter(|name| !rules.iter().any(|r| &r.name == *name))
            .collect();
        if !unknown.is_empty() {
            for name in unknown {
                eprintln!("error: unknown rule '{name}'");
            }
            return 2;
        }
        rules.retain(|r| args.rule.contains(&r.name));
    }

    let mut results: Vec<SourceResult> = Vec::new();
    let mut visited: std::collections::HashSet<std::path::PathBuf> = std::collections::HashSet::new();

    if let Some(expr) = &args.expression {
        results.push(analyze(expr.clone(), "-e".to_string(), &rules));
    } else if args.paths.is_empty() {
        match read_stdin() {
            Ok(src) => results.push(analyze(src, "-".to_string(), &rules)),
            Err(e) => {
                eprintln!("error reading stdin: {e}");
                return 2;
            }
        }
    } else {
        for path in &args.paths {
            if path.is_dir() {
                let mut found = false;
                for entry in WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        let ext = e.path().extension().and_then(|s| s.to_str());
                        ext == Some("cypher") || (!args.no_markdown && ext == Some("md"))
                    })
                {
                    let canonical = entry.path().canonicalize().unwrap_or_else(|_| entry.path().to_path_buf());
                    if !visited.insert(canonical) { continue; }
                    found = true;
                    let ext = entry.path().extension().and_then(|s| s.to_str());
                    if ext == Some("md") {
                        match lint_markdown_file(entry.path(), &rules) {
                            Ok(r) => results.push(r),
                            Err(e) => {
                                eprintln!("{}: {e}", entry.path().display());
                                return 2;
                            }
                        }
                    } else {
                        match std::fs::read_to_string(entry.path()) {
                            Ok(src) => results.push(analyze(
                                src,
                                entry.path().display().to_string(),
                                &rules,
                            )),
                            Err(e) => {
                                eprintln!("{}: {e}", entry.path().display());
                                return 2;
                            }
                        }
                    }
                }
                if !found {
                    if args.no_markdown {
                        eprintln!("note: no .cypher files found in {}", path.display());
                    } else {
                        eprintln!("note: no .cypher or .md files found in {}", path.display());
                    }
                }
            } else {
                let ext = path.extension().and_then(|s| s.to_str());
                if args.no_markdown && ext == Some("md") {
                    eprintln!("note: {}: skipped (--no-markdown)", path.display());
                    continue;
                }
                let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
                if !visited.insert(canonical) { continue; }
                if ext == Some("md") {
                    match lint_markdown_file(path, &rules) {
                        Ok(r) => results.push(r),
                        Err(e) => {
                            eprintln!("{}: {e}", path.display());
                            return 2;
                        }
                    }
                } else {
                    match std::fs::read_to_string(path) {
                        Ok(src) => results.push(analyze(src, path.display().to_string(), &rules)),
                        Err(e) => {
                            eprintln!("{}: {e}", path.display());
                            return 2;
                        }
                    }
                }
            }
        }
    }

    let has_errors = results
        .iter()
        .any(|r| r.diags.iter().any(|d| matches!(d.severity, Severity::Error)));
    let has_warnings = results
        .iter()
        .any(|r| r.diags.iter().any(|d| matches!(d.severity, Severity::Warning)));

    if args.json {
        let result = LintResult {
            schema_version: 1,
            tool: format!("cypher/{}", env!("CARGO_PKG_VERSION")),
            files: results
                .iter()
                .map(|r| FileResult { path: r.path.clone(), diagnostics: r.diags.clone() })
                .collect(),
        };
        match serde_json::to_string_pretty(&result) {
            Ok(json) => println!("{json}"),
            Err(e) => {
                eprintln!("error serializing JSON: {e}");
                return 2;
            }
        }
    } else {
        print_pretty(&results);
    }

    if has_errors || (args.strict && has_warnings) { 1 } else { 0 }
}

fn analyze(source: String, path: String, rules: &[Rule]) -> SourceResult {
    let cypher_lang: tree_sitter::Language = tree_sitter_cypher::LANGUAGE.into();
    let cypherdoc_lang: tree_sitter::Language = tree_sitter_cypherdoc::LANGUAGE.into();

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&cypher_lang).expect("cypher parser");
    let tree = parser.parse(&source, None).expect("parse");

    let mut diags: Vec<Diagnostic> = Vec::new();

    if tree.root_node().has_error() {
        collect_error_nodes(tree.root_node(), &source, &mut diags);
    }

    let pairs = collect_pairs(&tree, &source);

    // Structural rules
    for rule in rules.iter().filter(|r| r.applies_to == AppliesTo::Structural) {
        let mut cursor = tree_sitter::QueryCursor::new();
        let mut qm = cursor.matches(&rule.query, tree.root_node(), source.as_bytes());
        while let Some(m) = qm.next() {
            let node = m.captures[0].node;
            diags.push(make_diagnostic(rule, node, &source, 0));
        }
    }

    // Contract and cross-reference rules — per pair
    for pair in &pairs {
        let (doc_tree, doc_source) = if let (Some(start), Some(end)) =
            (pair.doc_start_byte, pair.doc_end_byte)
        {
            let doc_src = source[start..end].to_string();
            let mut doc_parser = tree_sitter::Parser::new();
            doc_parser.set_language(&cypherdoc_lang).expect("cypherdoc parser");
            let dt = doc_parser.parse(&doc_src, None).expect("parse doc");
            (Some(dt), Some((doc_src, start)))
        } else {
            (None, None)
        };

        // Contract rules
        if let (Some(ref dt), Some((ref doc_src, doc_start))) = (&doc_tree, &doc_source) {
            let has_content = dt.root_node().child_count() > 0
                && dt.root_node().children(&mut dt.root_node().walk()).any(|c| c.is_named());

            if has_content {
                for rule in rules.iter().filter(|r| r.applies_to == AppliesTo::Contract) {
                    let mut cursor = tree_sitter::QueryCursor::new();
                    let mut qm =
                        cursor.matches(&rule.query, dt.root_node(), doc_src.as_bytes());
                    while let Some(m) = qm.next() {
                        let node = m.captures[0].node;
                        diags.push(make_diagnostic(rule, node, &source, *doc_start));
                    }
                }

                // MissingToolName: the grammar uses a MISSING zero-width node when name
                // is absent. Check that the first named child is "name" AND non-empty.
                let missing_name = dt
                    .root_node()
                    .named_child(0)
                    .map_or(true, |n| n.kind() != "name" || n.start_byte() == n.end_byte());
                if missing_name {
                    let (line, col) = byte_to_line_col(&source, pair.doc_start_byte.unwrap_or(0));
                    diags.push(Diagnostic {
                        severity: Severity::Warning,
                        rule: "MissingToolName".to_string(),
                        message: "Cypherdoc comment has no tool name. Add a name as the first line.".to_string(),
                        range: Range {
                            start: Position { line: line as u32, character: col as u32 },
                            end: Position { line: line as u32, character: col as u32 },
                        },
                        code: None,
                    });
                }
            }
        }

        // Cross-reference rules
        let has_cross_ref = rules.iter().any(|r| r.applies_to == AppliesTo::CrossReference);
        if has_cross_ref {
            let stmt_src = &source[pair.statement_start_byte..pair.statement_end_byte];
            let stmt_offset = pair.statement_start_byte;

            // Collect declared params from cypherdoc tree
            let declared: HashSet<String> =
                if let (Some(ref dt), Some((ref doc_src, _))) = (&doc_tree, &doc_source) {
                    if let Some(rule) =
                        rules.iter().find(|r| r.name == "UnusedParameter")
                    {
                        let mut names = HashSet::new();
                        let mut cursor = tree_sitter::QueryCursor::new();
                        let mut qm =
                            cursor.matches(&rule.query, dt.root_node(), doc_src.as_bytes());
                        while let Some(m) = qm.next() {
                            for cap in m.captures {
                                names.insert(doc_src[cap.node.byte_range()].to_string());
                            }
                        }
                        names
                    } else {
                        HashSet::new()
                    }
                } else {
                    HashSet::new()
                };

            // Collect used params from Cypher statement
            let (used_nodes, used): (Vec<_>, HashSet<String>) = {
                let mut nodes = Vec::new();
                let mut names = HashSet::new();
                if let Some(rule) =
                    rules.iter().find(|r| r.name == "UndocumentedParameter")
                {
                    // Re-parse just the statement slice for accurate byte offsets
                    let mut stmt_parser = tree_sitter::Parser::new();
                    stmt_parser
                        .set_language(&cypher_lang)
                        .expect("cypher parser");
                    let stmt_tree =
                        stmt_parser.parse(stmt_src, None).expect("parse stmt");
                    let mut cursor = tree_sitter::QueryCursor::new();
                    let mut qm = cursor.matches(
                        &rule.query,
                        stmt_tree.root_node(),
                        stmt_src.as_bytes(),
                    );
                    while let Some(m) = qm.next() {
                        for cap in m.captures {
                            let text =
                                stmt_src[cap.node.byte_range()].trim_start_matches('$').to_string();
                            nodes.push((cap.node.start_byte() + stmt_offset, cap.node.end_byte() + stmt_offset));
                            names.insert(text);
                        }
                    }
                }
                (nodes, names)
            };

            if doc_tree.is_some() {
                // UnusedParameter: declared but not used
                for unused in declared.difference(&used) {
                    if let (Some(ref dt), Some((ref doc_src, doc_start))) =
                        (&doc_tree, &doc_source)
                    {
                        if let Some(rule) =
                            rules.iter().find(|r| r.name == "UnusedParameter")
                        {
                            let mut cursor = tree_sitter::QueryCursor::new();
                            let mut qm = cursor.matches(
                                &rule.query,
                                dt.root_node(),
                                doc_src.as_bytes(),
                            );
                            while let Some(m) = qm.next() {
                                for cap in m.captures {
                                    if &doc_src[cap.node.byte_range()] == unused {
                                        let mut d = make_diagnostic(rule, cap.node, &source, *doc_start);
                                        d.message = format!(
                                            "@param \"{unused}\" is declared but ${unused} never appears in the query."
                                        );
                                        diags.push(d);
                                    }
                                }
                            }
                        }
                    }
                }

                // UndocumentedParameter: used but not declared
                for (start, end) in &used_nodes {
                    let param_text =
                        source[*start..*end].trim_start_matches('$').to_string();
                    if !declared.contains(&param_text) {
                        let (line, col) = byte_to_line_col(&source, *start);
                        let (end_line, end_col) = byte_to_line_col(&source, *end);
                        if let Some(rule) =
                            rules.iter().find(|r| r.name == "UndocumentedParameter")
                        {
                            diags.push(Diagnostic {
                                severity: rule.severity.clone(),
                                rule: rule.name.clone(),
                                message: format!(
                                    "${param_text} is used in the query but not declared as @param in the doc comment."
                                ),
                                range: Range {
                                    start: Position {
                                        line: line as u32,
                                        character: col as u32,
                                    },
                                    end: Position {
                                        line: end_line as u32,
                                        character: end_col as u32,
                                    },
                                },
                                code: None,
                            });
                        }
                    }
                }
            }
        }
    }

    SourceResult { path, source, diags }
}

fn lint_markdown_file(path: &Path, rules: &[Rule]) -> anyhow::Result<SourceResult> {
    let full_source = std::fs::read_to_string(path)?;
    let path_str = path.display().to_string();
    let snippets = extract_cypher_snippets(&full_source);
    let mut all_diags: Vec<Diagnostic> = Vec::new();
    for snippet in snippets {
        if !snippet.closed {
            // start_line is 0-based content; fence opening is one line earlier = 1-based start_line
            eprintln!("note: {path_str}: unclosed ```cypher fence at line {}", snippet.start_line);
        }
        if snippet.content.trim().is_empty() {
            continue;
        }
        let mut result = analyze(snippet.content.clone(), path_str.clone(), rules);
        for d in &mut result.diags {
            d.range.start.line += snippet.start_line;
            d.range.end.line += snippet.start_line;
        }
        all_diags.extend(result.diags);
    }
    Ok(SourceResult { path: path_str, source: full_source, diags: all_diags })
}

fn collect_pairs(tree: &Tree, _source: &str) -> Vec<DocStatementPair> {
    let root = tree.root_node();
    let mut pairs = Vec::new();
    let mut cursor = root.walk();
    let children: Vec<Node<'_>> = root.children(&mut cursor).collect();
    let mut i = 0;
    while i < children.len() {
        let child = children[i];
        if child.kind() == "doc_comment" && i + 1 < children.len() && children[i + 1].kind() == "statement" {
            pairs.push(DocStatementPair {
                doc_start_byte: Some(child.start_byte()),
                doc_end_byte: Some(child.end_byte()),
                statement_start_byte: children[i + 1].start_byte(),
                statement_end_byte: children[i + 1].end_byte(),
            });
            i += 2;
        } else if child.kind() == "statement" {
            pairs.push(DocStatementPair {
                doc_start_byte: None,
                doc_end_byte: None,
                statement_start_byte: child.start_byte(),
                statement_end_byte: child.end_byte(),
            });
            i += 1;
        } else {
            i += 1;
        }
    }
    pairs
}

fn collect_error_nodes(node: Node<'_>, source: &str, diags: &mut Vec<Diagnostic>) {
    if node.is_error() || node.is_missing() {
        let (line, col) = byte_to_line_col(source, node.start_byte());
        let (end_line, end_col) = byte_to_line_col(source, node.end_byte());
        diags.push(Diagnostic {
            severity: Severity::Error,
            rule: "ParseError".to_string(),
            message: "Syntax error in Cypher query.".to_string(),
            range: Range {
                start: Position { line: line as u32, character: col as u32 },
                end: Position { line: end_line as u32, character: end_col as u32 },
            },
            code: None,
        });
        return;
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        collect_error_nodes(child, source, diags);
    }
}

// `full_source` is always the complete file text; `byte_offset` is the byte position
// of `node`'s containing slice within `full_source` (0 for nodes already in full_source).
fn make_diagnostic(rule: &Rule, node: Node<'_>, full_source: &str, byte_offset: usize) -> Diagnostic {
    let abs_start = (node.start_byte() + byte_offset).min(full_source.len());
    let abs_end = (node.end_byte() + byte_offset).min(full_source.len());
    let (line, col) = byte_to_line_col(full_source, abs_start);
    let (end_line, end_col) = byte_to_line_col(full_source, abs_end);
    Diagnostic {
        severity: rule.severity.clone(),
        rule: rule.name.clone(),
        message: rule.message.clone(),
        range: Range {
            start: Position { line: line as u32, character: col as u32 },
            end: Position { line: end_line as u32, character: end_col as u32 },
        },
        code: rule.code.clone(),
    }
}

// Returns (zero-based line, UTF-16 code-unit column) — matches LSP Position.
fn byte_to_line_col(s: &str, byte: usize) -> (usize, usize) {
    let byte = byte.min(s.len());
    let byte = (0..=byte).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0);
    let prefix = &s[..byte];
    let line = prefix.matches('\n').count();
    let line_start = prefix.rfind('\n').map(|p| p + 1).unwrap_or(0);
    // Count UTF-16 code units from the start of the line.
    let col = prefix[line_start..].chars().map(|c| if c as u32 > 0xFFFF { 2 } else { 1 }).sum();
    (line, col)
}

fn run_tree(args: &LintArgs) -> i32 {
    if args.paths.len() > 1 {
        eprintln!("error: --tree accepts at most one input");
        return 2;
    }
    let src = if let Some(expr) = &args.expression {
        expr.clone()
    } else if args.paths.is_empty() {
        match read_stdin() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error reading stdin: {e}");
                return 2;
            }
        }
    } else {
        match std::fs::read_to_string(&args.paths[0]) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}: {e}", args.paths[0].display());
                return 2;
            }
        }
    };

    let lang: tree_sitter::Language = tree_sitter_cypher::LANGUAGE.into();
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang).expect("cypher parser");
    let tree = parser.parse(&src, None).expect("parse");
    println!("{}", tree.root_node().to_sexp());
    0
}

fn print_pretty(results: &[SourceResult]) {
    for r in results {
        for d in &r.diags {
            let kind = match d.severity {
                Severity::Error => ReportKind::Error,
                Severity::Warning => ReportKind::Warning,
                Severity::Information | Severity::Hint => ReportKind::Advice,
            };
            let color = match d.severity {
                Severity::Error => Color::Red,
                Severity::Warning => Color::Yellow,
                Severity::Information | Severity::Hint => Color::Cyan,
            };

            let start_byte = line_col_to_byte(
                &r.source,
                d.range.start.line as usize,
                d.range.start.character as usize,
            );
            let end_byte = line_col_to_byte(
                &r.source,
                d.range.end.line as usize,
                d.range.end.character as usize,
            )
            .max(start_byte + 1);
            let start_char = byte_to_char(&r.source, start_byte);
            let end_char = byte_to_char(&r.source, end_byte).max(start_char + 1);

            Report::build(kind, (r.path.clone(), start_char..end_char))
                .with_message(&d.message)
                .with_label(
                    Label::new((r.path.clone(), start_char..end_char))
                        .with_message(&d.message)
                        .with_color(color),
                )
                .with_code(match &d.code {
                    Some(c) => format!("{}/{}", d.rule, c),
                    None => d.rule.clone(),
                }.as_str())
                .finish()
                .eprint(sources([(r.path.clone(), r.source.as_str())]))
                .ok();
        }
    }
}

// Inverse of `byte_to_line_col`: `col` is UTF-16 code units from line start.
fn line_col_to_byte(s: &str, line: usize, col_utf16: usize) -> usize {
    // Walk to the start of the requested line.
    let mut current_line = 0;
    let mut line_start = 0;
    for (i, ch) in s.char_indices() {
        if current_line == line { line_start = i; break; }
        if ch == '\n' { current_line += 1; }
        if i + ch.len_utf8() == s.len() { line_start = s.len(); }
    }
    // Advance col_utf16 UTF-16 code units from line_start.
    let mut units_remaining = col_utf16;
    for (byte_off, ch) in s[line_start..].char_indices() {
        if units_remaining == 0 { return line_start + byte_off; }
        units_remaining = units_remaining.saturating_sub(if ch as u32 > 0xFFFF { 2 } else { 1 });
    }
    s.len()
}

fn byte_to_char(s: &str, byte: usize) -> usize {
    let byte = byte.min(s.len());
    let byte = (0..=byte).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0);
    s[..byte].chars().count()
}

fn applies_to_header(src: &str) -> &str {
    for line in src.lines() {
        if let Some(rest) = line.strip_prefix(";; ") {
            if let Some(v) = rest.strip_prefix("Applies-to: ") {
                return v.trim();
            }
        }
    }
    "structural"
}

fn read_stdin() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}
