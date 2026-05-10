use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
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

// ── Polish: edge cases + unclosed fence ─────────────────────────────────────

#[test]
fn markdown_unclosed_fence_emits_note() {
    cypher()
        .args(["lint", fixture("markdown_unclosed_fence.md").to_str().unwrap()])
        .assert()
        .success()
        .stderr(contains("unclosed ```cypher fence"));
}

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

// ── Phase 2: Code field infrastructure (US5) ─────────────────────────────────

#[test]
fn rule_with_code_header_emits_code_in_json() {
    // CartesianProduct has Code: 03N90 — verify it appears in JSON output
    cypher()
        .args(["lint", "--json", "-e", "MATCH (a:A), (b:B) RETURN a, b"])
        .assert()
        .success()
        .stdout(contains("\"code\": \"03N90\""));
}

#[test]
fn rule_without_code_header_omits_code_from_json() {
    // UnlabelledNode has no Code: header — verify "code" key is absent from its diagnostic
    cypher()
        .args(["lint", "--json", "-e", "MATCH (n) RETURN n"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"code\"").not());
}

// ── Phase 3: US1 — CartesianProduct ──────────────────────────────────────────

#[test]
fn cartesian_product_warns_on_disconnected_match() {
    cypher()
        .args(["lint", "-e", "MATCH (a:User), (b:Order) RETURN a, b"])
        .assert()
        .success()
        .stderr(contains("CartesianProduct"));
}

#[test]
fn cartesian_product_clean_on_connected_match() {
    cypher()
        .args(["lint", "-e", "MATCH (a:User)-[:PLACED]->(b:Order) RETURN a, b"])
        .assert()
        .success()
        .stderr(predicates::str::contains("CartesianProduct").not());
}

#[test]
fn cartesian_product_three_patterns_two_warnings() {
    // Three disconnected patterns → two CartesianProduct warnings (second and third)
    let output = cypher()
        .args(["lint", "--json", "-e", "MATCH (a:A), (b:B), (c:C) RETURN a, b, c"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json = std::str::from_utf8(&output).unwrap();
    let count = json.matches("CartesianProduct").count();
    assert_eq!(count, 2, "expected 2 CartesianProduct diagnostics, got {count}");
}

// ── Phase 4: US2 — DeprecatedFunction (id()) ─────────────────────────────────

#[test]
fn deprecated_id_warns_on_bare_id_call() {
    cypher()
        .args(["lint", "-e", "MATCH (n) RETURN id(n)"])
        .assert()
        .success()
        .stderr(contains("DeprecatedFunction"))
        .stderr(contains("elementId"));
}

#[test]
fn deprecated_id_clean_on_element_id() {
    cypher()
        .args(["lint", "-e", "MATCH (n) RETURN elementId(n)"])
        .assert()
        .success()
        .stderr(predicates::str::contains("DeprecatedFunction").not());
}

#[test]
fn deprecated_id_fires_in_where_clause() {
    cypher()
        .args(["lint", "-e", "MATCH (n) WHERE id(n) > 0 RETURN n"])
        .assert()
        .success()
        .stderr(contains("DeprecatedFunction"));
}

#[test]
fn deprecated_id_fires_on_relationship() {
    cypher()
        .args(["lint", "-e", "MATCH ()-[r:REL]->() WHERE id(r) > 0 RETURN r"])
        .assert()
        .success()
        .stderr(contains("DeprecatedFunction"));
}

#[test]
fn deprecated_id_clean_on_qualified_name() {
    // apoc.id() is a qualified name — should NOT be flagged
    cypher()
        .args(["lint", "-e", "MATCH (n) RETURN apoc.id(n)"])
        .assert()
        .success()
        .stderr(predicates::str::contains("DeprecatedFunction").not());
}

// ── Phase 5: US3 — DynamicProperty ───────────────────────────────────────────

#[test]
fn dynamic_property_info_on_parameter_key() {
    cypher()
        .args(["lint", "-e", "MATCH (n) WHERE n[$key] IS NOT NULL RETURN n"])
        .assert()
        .success()
        .stderr(contains("DynamicProperty"));
}

#[test]
fn dynamic_property_clean_on_dot_access() {
    cypher()
        .args(["lint", "-e", "MATCH (n) RETURN n.name"])
        .assert()
        .success()
        .stderr(predicates::str::contains("DynamicProperty").not());
}

#[test]
fn dynamic_property_clean_on_integer_index() {
    cypher()
        .args(["lint", "-e", "MATCH (n) RETURN n[0]"])
        .assert()
        .success()
        .stderr(predicates::str::contains("DynamicProperty").not());
}

#[test]
fn dynamic_property_clean_on_string_literal_key() {
    cypher()
        .args(["lint", "-e", r#"MATCH (n) RETURN n["name"]"#])
        .assert()
        .success()
        .stderr(predicates::str::contains("DynamicProperty").not());
}

#[test]
fn dynamic_property_fires_in_return_clause() {
    // Dynamic key in RETURN clause — valid Cypher, DynamicProperty fires regardless of clause context
    cypher()
        .args(["lint", "-e", "MATCH (n:Node) RETURN n[$key]"])
        .assert()
        .success()
        .stderr(contains("DynamicProperty"));
}

// ── Phase 6: US5 — JSON code field for each rule ─────────────────────────────

#[test]
fn cartesian_product_json_has_code_03n90() {
    cypher()
        .args(["lint", "--json", "-e", "MATCH (a:A), (b:B) RETURN a, b"])
        .assert()
        .success()
        .stdout(contains("\"code\": \"03N90\""));
}

#[test]
fn deprecated_id_json_has_code_01n01() {
    cypher()
        .args(["lint", "--json", "-e", "MATCH (n) RETURN id(n)"])
        .assert()
        .success()
        .stdout(contains("\"code\": \"01N01\""));
}

#[test]
fn dynamic_property_json_has_code_03n95() {
    cypher()
        .args(["lint", "--json", "-e", "MATCH (n) WHERE n[$key] IS NOT NULL RETURN n"])
        .assert()
        .success()
        .stdout(contains("\"code\": \"03N95\""));
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
