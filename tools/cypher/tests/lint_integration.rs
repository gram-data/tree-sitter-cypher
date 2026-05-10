use assert_cmd::Command;
use predicates::str::contains;

fn cypher() -> Command {
    Command::cargo_bin("cypher").unwrap()
}

fn fixture(name: &str) -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures");
    p.push(name);
    p
}

// ── US1: Single-file lint ────────────────────────────────────────────────────

#[test]
fn clean_file_exits_zero() {
    cypher().args(["lint", fixture("clean.cypher").to_str().unwrap()]).assert().success();
}

#[test]
fn unlabelled_node_warns() {
    // UnlabelledNode is Warning severity — exits 0, but warning appears in stderr
    cypher()
        .args(["lint", fixture("unlabelled_node.cypher").to_str().unwrap()])
        .assert()
        .success()
        .stderr(contains("UnlabelledNode"));
}

#[test]
fn unbounded_relationship_exits_one() {
    cypher()
        .args(["lint", fixture("unbounded_relationship.cypher").to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("UnboundedRelationship"));
}

#[test]
fn unused_param_warns() {
    // UnusedParameter is Warning severity — exits 0
    cypher()
        .args(["lint", fixture("unused_param.cypher").to_str().unwrap()])
        .assert()
        .success()
        .stderr(contains("UnusedParameter"));
}

#[test]
fn undocumented_param_warns() {
    // UndocumentedParameter is Warning severity — exits 0
    cypher()
        .args(["lint", fixture("undocumented_param.cypher").to_str().unwrap()])
        .assert()
        .success()
        .stderr(contains("UndocumentedParameter"));
}

#[test]
fn optional_param_error_exits_one() {
    // OptionalParamMissingDefault is Error severity — exits 1
    cypher()
        .args(["lint", fixture("optional_param_error.cypher").to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("OptionalParamMissingDefault"));
}

#[test]
fn missing_tool_name_warns() {
    // MissingToolName is Warning severity — exits 0
    cypher()
        .args(["lint", fixture("missing_tool_name.cypher").to_str().unwrap()])
        .assert()
        .success()
        .stderr(contains("MissingToolName"));
}

#[test]
fn parse_error_exits_one_without_panic() {
    cypher()
        .args(["lint", fixture("parse_error.cypher").to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("ParseError"));
}

#[test]
fn empty_doc_produces_no_contract_warnings() {
    cypher()
        .args(["lint", fixture("empty_doc.cypher").to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn json_output_has_schema_version() {
    // Warning exits 0; JSON still emitted to stdout
    cypher()
        .args(["lint", "--json", fixture("unlabelled_node.cypher").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("\"schema_version\": 1"));
}

#[test]
fn strict_exits_one_on_warnings_only() {
    cypher()
        .args(["lint", "--strict", fixture("unlabelled_node.cypher").to_str().unwrap()])
        .assert()
        .failure()
        .code(1);
}

// ── US2: Batch / directory lint ──────────────────────────────────────────────

#[test]
fn directory_mode_exits_nonzero() {
    let fixtures_dir = fixture(".");
    cypher().args(["lint", fixtures_dir.to_str().unwrap()]).assert().failure().code(1);
}

#[test]
fn empty_directory_exits_zero_with_note() {
    let dir = tempfile::tempdir().unwrap();
    cypher()
        .args(["lint", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stderr(contains("no .cypher or .md files found"));
}

#[test]
fn stdin_mode_detects_unlabelled_node() {
    // Warning exits 0 but appears in stderr
    cypher()
        .args(["lint"])
        .write_stdin("MATCH (n) RETURN n\n")
        .assert()
        .success()
        .stderr(contains("UnlabelledNode"));
}

// ── US3: Parse tree inspection ───────────────────────────────────────────────

#[test]
fn tree_flag_outputs_sexp() {
    cypher()
        .args(["lint", "--tree", fixture("clean.cypher").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("(source_file"));
}

#[test]
fn tree_and_json_together_exit_two() {
    cypher()
        .args(["lint", "--tree", "--json", fixture("clean.cypher").to_str().unwrap()])
        .assert()
        .code(2);
}

// ── US1 (markdown): Single markdown file lint ────────────────────────────────

#[test]
fn markdown_clean_exits_zero() {
    cypher()
        .args(["lint", fixture("markdown_clean.md").to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn markdown_unlabelled_warns() {
    // UnlabelledNode is Warning — exits 0 but appears in stderr
    cypher()
        .args(["lint", fixture("markdown_unlabelled.md").to_str().unwrap()])
        .assert()
        .success()
        .stderr(contains("UnlabelledNode"));
}

#[test]
fn markdown_unlabelled_json_has_correct_line() {
    // MATCH (n) is at 0-based markdown line 8; JSON uses 0-based lines
    cypher()
        .args(["lint", "--json", fixture("markdown_unlabelled.md").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("\"line\": 8"));
}

#[test]
fn markdown_unlabelled_json_has_md_path() {
    cypher()
        .args(["lint", "--json", fixture("markdown_unlabelled.md").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("markdown_unlabelled.md"));
}

#[test]
fn markdown_multi_snippet_reports_both_rules() {
    // The multi-snippet file has UnlabelledNode (warning) and UnboundedRelationship (error)
    cypher()
        .args(["lint", fixture("markdown_multi_snippet.md").to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("UnlabelledNode"))
        .stderr(contains("UnboundedRelationship"));
}

#[test]
fn markdown_multi_snippet_json_has_offset_lines() {
    // Snippet 2 content starts at 0-based line 15; snippet 3 at line 24
    cypher()
        .args(["lint", "--json", fixture("markdown_multi_snippet.md").to_str().unwrap()])
        .assert()
        .failure()
        .stdout(contains("\"line\": 15"))
        .stdout(contains("\"line\": 24"));
}

// ── US2 (markdown): Directory scan includes .md files ────────────────────────

#[test]
fn directory_includes_md_files() {
    // fixtures/ has both .cypher and .md files; at least one .md has an error
    let fixtures_dir = fixture(".");
    cypher()
        .args(["lint", fixtures_dir.to_str().unwrap()])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn markdown_no_fence_exits_zero() {
    cypher()
        .args(["lint", fixture("markdown_no_fence.md").to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn empty_directory_note_without_no_markdown() {
    let dir = tempfile::tempdir().unwrap();
    cypher()
        .args(["lint", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stderr(contains("no .cypher or .md files found"));
}

// ── US3 (markdown): --no-markdown flag ───────────────────────────────────────

#[test]
fn no_markdown_skips_md_in_directory() {
    // fixtures/ contains .md files with warnings/errors; --no-markdown should
    // produce results only from .cypher files (which still have errors)
    let fixtures_dir = fixture(".");
    cypher()
        .args(["lint", "--no-markdown", fixtures_dir.to_str().unwrap()])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn no_markdown_with_explicit_md_path_exits_zero() {
    cypher()
        .args(["lint", "--no-markdown", fixture("markdown_unlabelled.md").to_str().unwrap()])
        .assert()
        .success()
        .stderr(contains("skipped (--no-markdown)"));
}

#[test]
fn no_markdown_empty_dir_note_uses_cypher_only_message() {
    let dir = tempfile::tempdir().unwrap();
    cypher()
        .args(["lint", "--no-markdown", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stderr(contains("no .cypher files found"));
}

// ── Polish: edge cases ───────────────────────────────────────────────────────

#[test]
fn markdown_empty_snippet_exits_zero() {
    cypher()
        .args(["lint", fixture("markdown_empty_snippet.md").to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn markdown_json_schema_version_present() {
    cypher()
        .args(["lint", "--json", fixture("markdown_unlabelled.md").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("\"schema_version\": 1"))
        .stdout(contains("\"tool\": \"cypher/"));
}

// ── US4: External dispatch ───────────────────────────────────────────────────

#[test]
fn unknown_subcommand_exits_two() {
    cypher()
        .args(["bogus-unknown-cmd"])
        .assert()
        .code(2)
        .stderr(contains("unknown sub-command"));
}
