use tree_sitter::Language;

use crate::types::Severity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppliesTo {
    Structural,
    Contract,
    CrossReference,
}

pub struct Rule {
    pub name: String,
    pub severity: Severity,
    pub applies_to: AppliesTo,
    pub message: String,
    pub query: tree_sitter::Query,
}

pub fn parse_rule_file(src: &str, language: Language) -> Result<Rule, String> {
    let mut name = None;
    let mut severity = None;
    let mut applies_to = None;
    let mut message = None;
    let mut query_lines: Vec<&str> = Vec::new();
    let mut in_query = false;

    for line in src.lines() {
        if !in_query {
            if let Some(rest) = line.strip_prefix(";; ") {
                if let Some(v) = rest.strip_prefix("Rule: ") {
                    name = Some(v.trim().to_string());
                } else if let Some(v) = rest.strip_prefix("Severity: ") {
                    severity = Some(match v.trim() {
                        "Error" => Severity::Error,
                        "Warning" => Severity::Warning,
                        "Information" => Severity::Information,
                        "Hint" => Severity::Hint,
                        other => return Err(format!("unknown severity: {other}")),
                    });
                } else if let Some(v) = rest.strip_prefix("Applies-to: ") {
                    applies_to = Some(match v.trim() {
                        "structural" => AppliesTo::Structural,
                        "contract" => AppliesTo::Contract,
                        "cross-reference" => AppliesTo::CrossReference,
                        other => return Err(format!("unknown applies-to: {other}")),
                    });
                } else if let Some(v) = rest.strip_prefix("Message: ") {
                    message = Some(v.trim().to_string());
                }
            } else if line.strip_prefix(";;").is_some() || line.trim().is_empty() {
                // bare ";;" comment or blank line before query starts — skip
            } else {
                in_query = true;
                query_lines.push(line);
            }
        } else {
            query_lines.push(line);
        }
    }

    let name = name.ok_or("missing Rule: header")?;
    let severity = severity.ok_or("missing Severity: header")?;
    let applies_to = applies_to.ok_or("missing Applies-to: header")?;
    let message = message.ok_or("missing Message: header")?;
    let query_src = query_lines.join("\n");
    let query = tree_sitter::Query::new(&language, &query_src)
        .map_err(|e| format!("query compile error in rule '{name}': {e}"))?;

    Ok(Rule { name, severity, applies_to, message, query })
}

pub fn builtin_rules() -> Vec<Rule> {
    let cypher_lang: Language = tree_sitter_cypher::LANGUAGE.into();
    let cypherdoc_lang: Language = tree_sitter_cypherdoc::LANGUAGE.into();

    let structural_sources: &[&str] = &[
        include_str!("../rules/structural/unlabelled_node.scm"),
        include_str!("../rules/structural/unbounded_relationship.scm"),
    ];
    let contract_sources: &[&str] = &[
        include_str!("../rules/contract/optional_param_missing_default.scm"),
    ];
    // Cross-ref rules are split: UndocumentedParameter queries the Cypher tree (parameter nodes),
    // UnusedParameter queries the cypherdoc tree (param_tag nodes).
    let cross_ref_cypher_sources: &[&str] = &[
        include_str!("../rules/cross_reference/undocumented_parameter.scm"),
    ];
    let cross_ref_cypherdoc_sources: &[&str] = &[
        include_str!("../rules/cross_reference/unused_parameter.scm"),
    ];

    let mut rules = Vec::new();

    for src in structural_sources {
        match parse_rule_file(src, cypher_lang.clone()) {
            Ok(r) => rules.push(r),
            Err(e) => eprintln!("warning: failed to load structural rule: {e}"),
        }
    }
    for src in contract_sources {
        match parse_rule_file(src, cypherdoc_lang.clone()) {
            Ok(r) => rules.push(r),
            Err(e) => eprintln!("warning: failed to load contract rule: {e}"),
        }
    }
    for src in cross_ref_cypher_sources {
        match parse_rule_file(src, cypher_lang.clone()) {
            Ok(r) => rules.push(r),
            Err(e) => eprintln!("warning: failed to load cross-reference rule: {e}"),
        }
    }
    for src in cross_ref_cypherdoc_sources {
        match parse_rule_file(src, cypherdoc_lang.clone()) {
            Ok(r) => rules.push(r),
            Err(e) => eprintln!("warning: failed to load cross-reference rule: {e}"),
        }
    }

    rules
}
