# BNF Gap Analysis: openCypher Spec vs. tree-sitter-cypher

## Overview

The openCypher BNF (`references/openCypher/grammar/openCypher.bnf`) defines approximately 200 named productions.
`grammar.js` implements roughly 60 named rules, covering the core clause structure, expressions, and literals
well enough to parse the most common Cypher idioms. However, large swaths of the spec — particularly
path-search prefixes (GQL-style), data-update patterns (CREATE/MERGE/SET with their dedicated node/relationship
filler rules), label-expression sub-productions, the literal type hierarchy, numeric sub-types, and most
tokenization helpers — are either absent or collapsed into broader rules that skip intermediate named nodes.
Overall coverage of named BNF rules is approximately 30%; coverage of *behaviour* (what actually parses
correctly) is higher because grammar.js merges many thin wrapper rules into richer parent rules.

---

## Missing Rules

Rules with no corresponding grammar.js rule at all, grouped by semantic area.

### Program / Statement structure

| BNF rule | What it covers |
|---|---|
| `<program>` | Top-level entry; grammar.js uses `source_file` (non-standard name) |
| `<procedure_specification>` | Wraps a statement block; not named separately |
| `<statement_block>` | Single-statement block; not named separately |
| `<composite_statement>` | Linear statement + UNION chaining; merged into `union_statement` |
| `<composite_conjunction>` | `UNION [ALL]` connector node; not a named rule |
| `<linear_statement>` | Sequence of clauses + optional result; not named |
| `<primitive_statement>` | Query / update / call discriminator; not named |
| `<primitive_query_statement>` | MATCH / UNWIND / WITH discriminator; not named |
| `<primitive_data_update_statement>` | CREATE / MERGE / SET / REMOVE / DELETE discriminator; not named |
| `<primitive_result_statement>` | Thin wrapper around RETURN; not named |

### Pattern / path-search prefixes (GQL path modes)

These productions are entirely absent — grammar.js has no rules for any of the new GQL-style path search modes.

| BNF rule | What it covers |
|---|---|
| `<graph_pattern_binding_table>` | Wraps `<graph_pattern>` for MATCH; not named |
| `<graph_pattern>` | Pattern + optional WHERE; grammar uses `pattern` + separate `where_clause` |
| `<graph_pattern_where_clause>` | WHERE inside a graph pattern; not named |
| `<path_pattern_list>` | Comma-separated path patterns; inlined |
| `<path_variable_declaration>` | `var =` before a path; inlined into `path_pattern` |
| `<path_pattern_prefix>` | Wraps path-search prefix; absent |
| `<path_search_prefix>` | ALL / ANY / SHORTEST discriminator; absent |
| `<all_path_search>` | `ALL [PATH|PATHS]` path mode; absent |
| `<any_path_search>` | `ANY [n] [PATH|PATHS]` path mode; absent |
| `<shortest_path_search>` | Discriminator for all SHORTEST forms; absent |
| `<all_shortest_path_search>` | `ALL SHORTEST [PATH|PATHS]`; absent |
| `<any_shortest_path_search>` | `ANY SHORTEST [PATH|PATHS]`; absent |
| `<counted_shortest_path_search>` | `SHORTEST n [PATH|PATHS]`; absent |
| `<counted_shortest_group_search>` | `SHORTEST [n] [PATH|PATHS] GROUP|GROUPS`; absent |
| `<number_of_paths>` | Unsigned integer for path count; absent |
| `<number_of_groups>` | Unsigned integer for group count; absent |
| `<path_keywords>` | `PATH` or `PATHS` keyword set; absent |

### Path pattern expressions

| BNF rule | What it covers |
|---|---|
| `<path_pattern_expression>` | Path term or legacy shortest path; absent as named rule |
| `<legacy_shortest_path_pattern>` | `shortestPath(...)` / `allShortestPaths(...)` function-style syntax; absent |
| `<path_term>` | Sequence of path factors; absent |
| `<path_factor>` | Path primary or quantified path primary; absent |
| `<quantified_path_primary>` | `<path_primary> <graph_pattern_quantifier>`; absent |
| `<path_primary>` | Element pattern or parenthesized path; absent |
| `<parenthesized_path_pattern_expression>` | `([subpath =] <path> [WHERE expr])`; absent |
| `<subpath_variable_declaration>` | `var =` inside parenthesized path; absent |
| `<parenthesized_path_pattern_where_clause>` | WHERE inside parenthesized path; absent |
| `<simple_path_pattern>` | Node + (rel + node)+ — used in pattern comprehension and pattern expression; absent as named rule (inlined) |
| `<graph_pattern_quantifier>` | `* + {n} {n,m}` on a path primary; absent as named rule |
| `<fixed_quantifier>` | `{n}` exact quantifier; absent |
| `<general_quantifier>` | `{n,m}` range quantifier; absent |

### Element / node / relationship pattern sub-productions

| BNF rule | What it covers |
|---|---|
| `<element_pattern>` | Node or relationship discriminator; absent |
| `<node_pattern_filler>` | Variable + label + predicate inside `()`; inlined |
| `<element_pattern_predicate>` | WHERE clause or property spec inside a pattern element; absent |
| `<element_pattern_where_clause>` | WHERE inside a node/rel pattern; absent (grammar only supports WHERE on MATCH clause) |
| `<element_property_specification>` | `{ key: val }` inside a pattern element; absent as named rule (inlined as `property_map`) |
| `<property_key_value_pair_list>` | Comma-separated key-value pairs; absent as named rule |
| `<property_key_value_pair>` | Single `key: value`; grammar uses `property_key_value` (partial match) |
| `<full_relationship_pattern>` | Discriminator for left/right/bidir/undirected; absent |
| `<full_relationship_pointing_left>` | `<-[...]-` pattern; absent (inlined into `relationship_pattern`) |
| `<full_relationship_pointing_right>` | `-[...]->` pattern; absent (inlined) |
| `<full_relationship_left_or_right>` | `<-[...]->` bidir; absent (inlined) |
| `<full_relationship_any_direction>` | `-[...]-` undirected; absent (inlined) |
| `<relationship_pattern_filler>` | Variable + label + path length + predicate inside `[]`; absent as named rule (uses `relationship_body`) |
| `<lower_and_upper_bound_path_length>` | `n..m` bounds sub-rule; absent |
| `<lower_bound_path_length>` | Lower bound integer; absent |
| `<upper_bound_path_length>` | Upper bound integer; absent |
| `<fixed_path_length>` | Exact-length integer; absent |

### CREATE / MERGE dedicated pattern rules

The BNF defines separate "create" and "merge" versions of node/rel patterns with stricter constraints
(CREATE requires a relationship type; MERGE requires exactly one type, etc.). grammar.js reuses the
general `pattern` rule for both, losing those structural distinctions.

| BNF rule | What it covers |
|---|---|
| `<create_graph_pattern>` | CREATE-specific top-level pattern; absent |
| `<create_path_pattern_list>` | Comma-separated CREATE paths; absent |
| `<create_path_pattern>` | Single CREATE path; absent |
| `<create_node_pattern>` | Node in CREATE context; absent |
| `<create_relationship_pattern>` | Rel in CREATE (pointing left or right only — no undirected); absent |
| `<create_relationship_pointing_left>` | `<-[filler]-` for CREATE; absent |
| `<create_relationship_pointing_right>` | `-[filler]->` for CREATE; absent |
| `<create_node_pattern_filler>` | Variable + labels + properties for CREATE node; absent |
| `<create_relationship_pattern_filler>` | Variable + type + properties for CREATE rel; absent |
| `<create_node_label_and_property_set_specification>` | Labels and/or properties for CREATE node; absent |
| `<create_relationship_label_and_property_set_specification>` | Type and properties for CREATE rel; absent |
| `<create_node_label_set_specification>` | Label set in CREATE context; absent |
| `<create_relationship_label_specification>` | Single type in CREATE context; absent |
| `<create_element_property_specification>` | Property map or parameter for CREATE; absent |
| `<merge_graph_pattern>` | MERGE-specific top-level pattern; absent |
| `<merge_path_pattern>` | Single MERGE path; absent |
| `<merge_node_pattern>` | Node in MERGE context; absent |
| `<merge_relationship_pattern>` | Rel in MERGE (directed only); absent |
| `<merge_relationship_pointing_left>` | `<-[filler]-` for MERGE; absent |
| `<merge_relationship_pointing_right>` | `-[filler]->` for MERGE; absent |
| `<merge_node_pattern_filler>` | Variable + labels + properties for MERGE node; absent |
| `<merge_relationship_pattern_filler>` | Variable + type + properties for MERGE rel; absent |
| `<merge_node_label_and_property_set_specification>` | Labels and/or properties for MERGE node; absent |
| `<merge_relationship_label_and_property_set_specification>` | Type and properties for MERGE rel; absent |
| `<merge_node_label_set_specification>` | Label set in MERGE context; absent |
| `<merge_relationship_label_specification>` | Single type in MERGE context; absent |
| `<merge_element_property_specification>` | Property map for MERGE; absent |

### SET / REMOVE item sub-productions

| BNF rule | What it covers |
|---|---|
| `<set_item_list>` | Comma-separated set items; absent as named rule |
| `<set_all_properties_item>` | `n = expr` (replace all properties); absent as named rule |
| `<add_all_properties_item>` | `n += expr` (merge properties); absent as named rule |
| `<set_labels_item>` | `n:Label` (add labels); absent as named rule |
| `<set_property_item>` | `n.prop = expr` (set one property); absent as named rule |
| `<remove_item_list>` | Comma-separated remove items; absent as named rule |
| `<remove_labels_item>` | `n:Label` (remove labels); absent as named rule |
| `<remove_property_item>` | `n.prop` (remove property); absent as named rule |

### DELETE sub-productions

| BNF rule | What it covers |
|---|---|
| `<delete_item_list>` | Comma-separated delete targets; absent as named rule |
| `<delete_item>` | Single expression to delete; absent as named rule |

### CALL / YIELD sub-productions

| BNF rule | What it covers |
|---|---|
| `<named_procedure_call>` | Procedure name + args + YIELD; absent as named rule |
| `<standalone_procedure_call>` | CALL without enclosing query; absent as named rule |
| `<explicit_procedure_arguments>` | `(arg, arg, ...)` wrapper; absent |
| `<procedure_argument_list>` | Comma-separated args; absent |
| `<procedure_argument>` | Single argument; absent |
| `<standalone_procedure_call_yield_clause>` | YIELD `*` or item list in standalone context; absent |
| `<yield_item_list>` | Comma-separated yield items; absent as named rule |
| `<yield_item_name>` | Field name component of yield item; absent |
| `<yield_item_alias>` | `AS var` component of yield item; absent |

### Result / RETURN sub-productions

| BNF rule | What it covers |
|---|---|
| `<return_statement>` | RETURN + body + order/page; absent as named rule |
| `<return_statement_body>` | DISTINCT + item list wrapper; absent as named rule |
| `<return_item_list>` | Comma-separated return items; absent as named rule |
| `<return_item_alias>` | `AS identifier` on a return item; absent as named rule |
| `<order_by_and_page_clause>` | ORDER BY + SKIP/OFFSET + LIMIT; absent (three separate rules in grammar.js) |
| `<sort_specification_list>` | Comma-separated sort specs; absent |
| `<sort_specification>` | Sort key + ordering spec; absent as named rule (grammar uses `sort_item`) |
| `<sort_key>` | Expression used as sort key; absent as named rule |
| `<ordering_specification>` | ASC/DESC discriminator wrapper; absent |
| `<ascending_order>` | ASC / ASCENDING; absent as named rule |
| `<descending_order>` | DESC / DESCENDING; absent as named rule |
| `<offset_clause>` | SKIP or OFFSET + expr; grammar uses `skip_clause` (non-standard name) |
| `<offset_synonym>` | SKIP / OFFSET keyword discriminator; absent |

### Label expression sub-productions

| BNF rule | What it covers |
|---|---|
| `<is_node_label_expression>` | `:label` or `IS label` on a node; inlined |
| `<is_relationship_label_expression>` | `:type` or `IS type` on a rel; inlined |
| `<is_label_expression>` | `(:` or `IS`) + label expr; absent as named rule |
| `<node_label_expression_legacy>` | `:A:B` colon-chain; inlined |
| `<relationship_label_expression_legacy>` | `:A\|:B` pipe-separated legacy rel types; absent |
| `<wildcard_label>` | `%` label wildcard; absent |
| `<label_expression>` | Top-level label expr (OR via `\|`); absent as named rule (uses `_label_expr_inner`) |
| `<label_term>` | AND layer in label expr; absent as named rule |
| `<label_factor>` | Negation or primary; absent as named rule |
| `<label_negation>` | `!label`; absent as named rule |
| `<label_primary>` | Name, parenthesized, or wildcard; absent as named rule |
| `<parenthesized_label_expression>` | `(label expr)` grouping; absent as named rule |
| `<node_label_set_specification_for_create_and_merge>` | `:A:B` for CREATE/MERGE; absent |
| `<node_label_set_specification_for_set_and_remove>` | `:A:B` for SET/REMOVE; absent |
| `<relationship_label_specification_for_create_and_merge>` | `:TYPE` for CREATE/MERGE; absent |

### Catalog references

| BNF rule | What it covers |
|---|---|
| `<procedure_reference>` | Qualified procedure name; absent (grammar uses `procedure_name`) |
| `<function_reference>` | Qualified function name; absent (grammar uses `function_name`) |
| `<catalog_object_parent_reference>` | Dot-separated namespace prefix; absent |
| `<object_name>` | Single namespace segment; absent |

### Expressions — intermediate layers

| BNF rule | What it covers |
|---|---|
| `<value_expression>` | Alias for boolean expression; absent as named rule |
| `<search_condition>` | Alias for boolean expression used in WHERE; absent |
| `<boolean_value_expression>` | OR layer; absent (merged into `binary_expression`) |
| `<boolean_term_xor>` | XOR layer; absent (merged) |
| `<boolean_term>` | AND layer; absent (merged) |
| `<boolean_factor>` | NOT + primary; absent (merged into `unary_expression`) |
| `<boolean_primary>` | Pattern or predicate; absent |
| `<predicate>` | Alias for comparison predicate; absent |
| `<comparison_predicate>` | Comparison chain wrapper; absent |
| `<simple_comparison_predicand>` | LHS of simple comparison; absent |
| `<simple_comparison_predicate_part_2>` | RHS op + predicand; absent |
| `<simple_comp_op>` | `= <> < > <= >= =~` discriminator; absent |
| `<advanced_comparison_predicand>` | Wraps arithmetic expression; absent |
| `<advanced_comparison_predicate_part_2>` | CONTAINS / IN / regex / STARTS WITH / ENDS WITH / IS NULL / IS labeled; absent |
| `<advanced_comp_op>` | CONTAINS / IN / =~ / STARTS WITH / ENDS WITH discriminator; absent |
| `<null_predicate_part_2>` | `IS [NOT] NULL`; absent as named rule |
| `<start_with_operator>` | `STARTS WITH`; absent as named rule |
| `<end_with_operator>` | `ENDS WITH`; absent as named rule |
| `<contains_operator>` | `CONTAINS`; absent as named rule |
| `<in_operator>` | `IN`; absent as named rule |
| `<is_labeled_predicate_part_2>` | IS label expression in predicate position; absent |
| `<arithmetic_value_expression>` | + / - layer; absent (merged into `binary_expression`) |
| `<arithmetic_term>` | * / / % layer; absent (merged) |
| `<arithmetic_factor>` | ^ power layer; absent (merged) |
| `<arithmetic_unary>` | Unary +/- + postfix; absent |
| `<postfix_expression>` | Base with property/subscript/slice postfix; absent as named rule |
| `<postfix_operator>` | Static property, dynamic element, or slice; absent |
| `<static_property_reference>` | `.property`; absent as named rule |
| `<dynamic_element_reference>` | `[expr]` subscript; absent as named rule |
| `<slicing>` | `[from..to]` slice; absent as named rule |
| `<slicing_from>` | Lower bound of slice; absent |
| `<slicing_to>` | Upper bound of slice; absent |
| `<non_parenthesized_value_expression_primary>` | Discriminator for all atom forms; absent |

### CASE expression sub-productions

| BNF rule | What it covers |
|---|---|
| `<simple_case>` | CASE expr WHEN ... END; absent as distinct named rule |
| `<search_case>` | CASE WHEN ... END (no operand); absent as distinct named rule |
| `<simple_when_clause>` | WHEN operand list THEN expr; absent |
| `<searched_when_clause>` | WHEN condition THEN expr; absent |
| `<case_operand>` | Expr between CASE and WHEN; absent |
| `<when_operand_list>` | Comma-separated operands; absent |
| `<when_operand>` | Single WHEN operand; absent |
| `<else_clause>` | ELSE expr; absent as named rule |

### Exists / subquery

| BNF rule | What it covers |
|---|---|
| `<subquery_expression_argument>` | Graph pattern or procedure spec; absent as named rule |

### Map projection

| BNF rule | What it covers |
|---|---|
| `<map_projection>` | `var { ... }` projection; absent |
| `<map_projection_element_list>` | Comma-separated projection elements; absent |
| `<map_projection_element>` | Literal field / field selector / variable / all-fields; absent |
| `<literal_map_field>` | `key: value` in projection; absent |
| `<field_selector>` | `.property` in projection; absent |
| `<variable_selector>` | Bare variable in projection; absent |
| `<all_fields_selector>` | `.*` in projection; absent |

### Comprehensions, reduce, quantifiers — sub-productions

| BNF rule | What it covers |
|---|---|
| `<list_element_source>` | `var IN expr` in list comprehension; absent as named rule |
| `<list_element_filter_and_projection>` | WHERE + pipe in list comprehension; absent |
| `<list_element_filter>` | WHERE clause inside comprehension; absent |
| `<list_element_projection>` | `| expr` inside comprehension; absent |
| `<pattern_source>` | `[var =] simple_path_pattern`; absent as named rule |
| `<pattern_filter_and_projection>` | WHERE + pipe in pattern comprehension; absent |
| `<pattern_filter>` | WHERE inside pattern comprehension; absent |
| `<pattern_projection>` | `| expr` inside pattern comprehension; absent |
| `<reduce_accumulator_initialization>` | `acc = init`; absent |
| `<reduce_element_source>` | `var IN list`; absent |
| `<reduce_step>` | `| expr`; absent |
| `<quantifier_expression>` | ALL/ANY/NONE/SINGLE wrapper; absent as named rule |
| `<quantifier>` | ALL / ANY / SINGLE / NONE discriminator; absent |
| `<quantifier_element_source>` | `var IN expr`; absent |
| `<quantifier_element_predicate>` | `WHERE expr`; absent |

### TRIM function

| BNF rule | What it covers |
|---|---|
| `<trim_function>` | TRIM(...) wrapper; absent |
| `<single_character_trim_function>` | `TRIM(expr)` form; absent |
| `<trim_source>` | Expr inside TRIM; absent |

### Shortest path expression (non-legacy)

| BNF rule | What it covers |
|---|---|
| `<shortest_path_expression>` | Wrapper aliasing `<legacy_shortest_path_pattern>`; absent |

### Value specifications and literals

| BNF rule | What it covers |
|---|---|
| `<value_specification>` | Literal / parameter / list constructor / map constructor; absent |
| `<list_value_constructor>` | `[expr, ...]` list in value context; absent |
| `<list_element_list>` | Comma-separated expressions in list constructor; absent |
| `<map_value_constructor>` | Map in value context; absent |
| `<record_value_constructor>` | Alias for fields specification; absent |
| `<fields_specification>` | `{ field list }`; absent |
| `<field_list>` | Comma-separated fields; absent |
| `<field>` | `name: value`; absent |
| `<unsigned_decimal_integer_specification>` | Integer or parameter; absent |
| `<literal>` | Signed numeric or general literal; absent as named rule |
| `<unsigned_literal>` | Unsigned numeric or general literal; absent |
| `<general_literal>` | Boolean / string / null / list / map; absent |
| `<signed_numeric_literal>` | `[-]` numeric or INF/INFINITY/NAN; absent (no INF/INFINITY/NAN support) |
| `<unsigned_numeric_literal>` | Exact or approximate numeric; absent |
| `<exact_numeric_literal>` | Integer (grammar splits into `integer_literal` / `float_literal`) |
| `<approximate_numeric_literal>` | Scientific notation / common notation + optional suffix; absent as named rule |
| `<approximate_number_suffix>` | F / D / f / d float type suffixes; absent |
| `<unsigned_decimal_in_scientific_notation>` | `mantissa E exponent`; absent as named rule |
| `<unsigned_decimal_in_common_notation>` | `integer.integer` or `.integer`; absent as named rule |
| `<mantissa>` | Common notation or integer; absent |
| `<exponent_indicator>` | E / e; absent |
| `<exponent>` | Signed decimal integer; absent |
| `<signed_decimal_integer>` | `[sign] unsigned decimal`; absent |
| `<unsigned_decimal_integer>` | Digit sequence with optional underscores; absent as named rule |
| `<unsigned_hexadecimal_integer>` | `0x` hex; absent as named rule (inlined into `integer_literal`) |
| `<unsigned_octal_integer>` | `0o` octal; absent as named rule (inlined) |
| `<list_literal>` (BNF version) | `[literal, ...]` (literals only, not arbitrary expressions); absent |
| `<list_element_list_literal>` | Comma-separated literals; absent |
| `<map_literal>` (BNF version) | Record literal; absent as named rule |
| `<record_literal>` | Fields literal; absent |
| `<fields_literal>` | `{ field list literal }`; absent |
| `<field_list_literal>` | Comma-separated field literals; absent |
| `<field_literal>` | `name: literal` (literal values only); absent |

### Names and variables

| BNF rule | What it covers |
|---|---|
| `<parameter_name>` | Separated identifier for parameter; absent |
| `<label_name>` | Identifier used as label; absent as named rule |
| `<property_name>` | Identifier used as property; absent as named rule |
| `<field_name>` | Identifier used as field; absent as named rule |
| `<binding_variable_reference>` | Variable reference (thin alias for variable); absent |
| `<binding_variable>` | Variable definition (thin alias for identifier); absent |
| `<procedure_name>` | Identifier used as procedure name; partially present |
| `<function_name>` | Identifier used as function name; present |
| `<object_name>` | Namespace segment identifier; absent |

### Tokenization helpers

These are low-level character / token definitions that Tree-sitter implements as regex terminals. None exist
as named rules, which is expected and acceptable, but they are listed for completeness.

`<token>`, `<non_delimiter_token>`, `<delimiter_token>`, `<separator>`, `<identifier_start>`,
`<identifier_extend>`, `<regular_identifier>`, `<extended_identifier>`, `<non_delimited_identifier>`,
`<separated_identifier>`, `<delimited_identifier>`, `<keyword>`, `<non_reserved_word>`,
`<special_character>`, all individual character terminals (`<ampersand>`, `<asterisk>`, etc.),
`<hex_digit>`, `<digit>`, `<standard_digit>`, `<octal_digit>`, `<binary_digit>`,
all escape sequence productions (`<escaped_character>`, `<escaped_reverse_solidus>`, etc.),
`<unicode_escape_value>`, `<unicode_4_digit_escape_value>`, `<unicode_6_digit_escape_value>`,
all operator token productions (`<concatenation_operator>`, `<double_colon>`, `<double_period>`,
`<regex_equals_operator>`, `<greater_than_operator>`, `<greater_than_or_equals_operator>`,
`<less_than_operator>`, `<less_than_or_equals_operator>`, `<not_equals_operator>`,
`<plus_equals>`, `<right_double_arrow>`),
string character sub-productions (`<single_quoted_character_representation>`,
`<double_quoted_character_representation>`, `<accent_quoted_character_representation>`,
`<double_single_quote>`, `<double_double_quote>`, `<double_grave_accent>`),
`<sign>` (inlined as `+`/`-` tokens), `<left_arrowhead>`, `<arrow_line>`, `<right_arrowhead>`.

---

## Partial / Stub Rules

Rules that exist in grammar.js but cover only part of what the BNF specifies.

| grammar.js rule | BNF counterpart | What's missing |
|---|---|---|
| `source_file` | `<program>` | Does not handle `<standalone_procedure_call>` as a top-level alternative |
| `union_statement` | `<composite_statement>` / `<composite_conjunction>` | `DISTINCT` set quantifier on UNION is not modelled |
| `statement` | `<linear_statement>` | Flat clause list rather than BNF's strict ordered structure; technically overly permissive |
| `match_clause` | `<simple_match_statement>` / `<optional_match_statement>` | Graph pattern WHERE is attached to `match_clause` but BNF places it inside `<graph_pattern>` |
| `node_pattern` | `<node_pattern>` + `<node_pattern_filler>` | `<element_pattern_where_clause>` (inline WHERE on a node) is not supported |
| `relationship_body` | `<relationship_pattern_filler>` | Inline WHERE on a relationship (`WHERE expr` inside `[...]`) is not supported |
| `path_pattern` | `<path_pattern>` | No path-search prefix support (ALL / ANY / SHORTEST modes); no `<parenthesized_path_pattern_expression>` |
| `path_length` | `<path_length>` + bounds sub-rules | Underscore separator in integers (e.g. `*1_000..2_000`) not supported; bounds are regex-only with no named nodes |
| `relationship_pattern` | `<full_relationship_pattern>` | Bidir `<->` (`<-[r]->`) is a Neo4j extension not in the BNF; missing `<element_pattern_where_clause>` inside `[]` |
| `label_expression` | `<is_node_label_expression>` | Legacy relationship label expression `:`A`\|:`B not supported; `%` wildcard label absent |
| `_label_expr_inner` | `<label_expression>` / `<label_term>` / `<label_factor>` / `<label_primary>` | Hidden rule — nodes not visible in parse tree; wildcard (`%`) absent |
| `expression` | `<value expression>` chain | INF / INFINITY / NAN literals absent; map projection (`var { ... }`) absent; `<shortest_path_expression>` / `<legacy_shortest_path_pattern>` absent |
| `binary_expression` | many BNF layers | String concatenation `\|\|` operator listed in BNF `<concatenation_operator>` but absorbed into `+` precedence level |
| `integer_literal` | `<unsigned_decimal_integer>` | Underscore digit separators (e.g. `1_000_000`) not supported |
| `float_literal` | `<approximate_numeric_literal>` | Float type suffixes `F`/`D`/`f`/`d` not supported; underscore digit separators not supported |
| `string_literal` | `<character_string_literal>` | Backtick-quoted strings (`<accent_quoted_character_sequence>`) only handled by `escaped_identifier`, not by string literal; `''` (doubled quote escape) not handled |
| `function_call` | `<function_invocation>` | `TRIM(expr)` should be handled separately as `<trim_function>` |
| `skip_clause` | `<offset_clause>` | Named `skip_clause` in grammar, `<offset_clause>` with synonym `SKIP\|OFFSET` in BNF — OFFSET keyword is correctly handled |
| `exists_expression` | `<exists_expression>` | `EXISTS { <graph_pattern> }` with graph-pattern WHERE is handled, but `<exists_expression>` BNF form uses `{}` while Neo4j often uses `()` — grammar is Neo4j-aligned, not spec-aligned |
| `pattern_predicate` | `<pattern_expression>` | No path-variable assignment before the pattern |
| `parameter` | `<general_parameter_reference>` | Parameter name restricted to identifiers and digits — spec allows `<separated_identifier>` (which includes extended identifiers) |
| `call_clause` | `<call_procedure_statement>` + `<standalone_procedure_call>` | Does not distinguish named vs. standalone call; missing `WHERE` after YIELD |
| `yield_clause` | `<yield_clause>` + `<standalone_procedure_call_yield_clause>` | `WHERE` after YIELD items (BNF: `<yield_clause> ::= YIELD <yield_item_list> [WHERE]`) not supported |
| `return_body` | `<return_statement_body>` | `ALL` set quantifier (in addition to `DISTINCT`) not supported |
| `property_key_value` | `<property_key_value_pair>` | Property key must be an `identifier` but BNF allows `<property_name>` which includes escaped identifiers |
| `map_literal` | `<map_value_constructor>` | Map keys restricted to plain identifiers; BNF allows `<field_name>` (which includes escaped identifiers) |
| `list_comprehension` | `<list_comprehension>` | `\| expr` projection is optional in grammar.js; BNF does allow the filter-only form so this is acceptable, but the grammar lists it as optional |
| `all_expression` / `any_expression` / `none_expression` / `single_expression` | `<quantifier_expression>` | WHERE clause is optional in grammar.js but BNF requires it (`<quantifier_element_predicate> ::= WHERE <value_expression>`) |

---

## Notable Gaps

High-impact missing features that would break or mangle common Cypher queries.

### 1. `shortestPath` and `allShortestPaths` — the trigger for this analysis

**Status**: ✅ **RESOLVED** in `008-opencypher-bnf-gap` — `legacy_shortest_path_pattern` rule added to `grammar.js`; `path_pattern` extended to accept it as an alternative body.

**BNF rules:** `<legacy_shortest_path_pattern>`, `<shortest_path_expression>`, `<path_pattern_expression>`

```
shortestPath((a)-[:KNOWS*]-(b))
allShortestPaths((a)-[*]-(b))
```

### 2. GQL-style path search prefixes (ALL / ANY / SHORTEST path modes)

**Status**: ✅ **RESOLVED** in `008-opencypher-bnf-gap` — `path_search_prefix` and all six sub-rules added; `match_clause` accepts the optional prefix.

**BNF rules:** `<path_search_prefix>`, `<all_path_search>`, `<any_path_search>`,
`<all_shortest_path_search>`, `<any_shortest_path_search>`,
`<counted_shortest_path_search>`, `<counted_shortest_group_search>`

```cypher
MATCH ALL (a)-[:KNOWS*]-(b)
MATCH ANY SHORTEST (a)-[*]-(b)
MATCH SHORTEST 3 (a)-[*]-(b)
MATCH SHORTEST 3 GROUPS (a)-[*]-(b)
```

### 3. Parenthesized path pattern expressions with quantifiers

**Status**: ✅ **RESOLVED** in `008-opencypher-bnf-gap` — `quantified_path_primary`, `graph_pattern_quantifier`, `fixed_quantifier`, `general_quantifier` added; `path_pattern` extended to allow quantified primaries.

**BNF rules:** `<quantified_path_primary>`,
`<graph_pattern_quantifier>`, `<fixed_quantifier>`, `<general_quantifier>`

```cypher
MATCH ((a)-[:KNOWS]->(b)){1,3}
MATCH ((a)-[*]->(b))+
```

### 4. Map projection

**Status**: ✅ **RESOLVED** in `008-opencypher-bnf-gap` — `map_projection`, `map_projection_element`, `field_selector`, `all_fields_selector`, `literal_map_field`, `variable_selector` added to `grammar.js`.

**BNF rule:** `<map_projection>`

```cypher
RETURN n { .name, .age, score: 10 }
RETURN n { .* }
```

### 5. Inline WHERE inside node and relationship patterns

**Status**: ✅ **RESOLVED** in `008-opencypher-bnf-gap` — `optional($.where_clause)` added as the last child of `node_pattern` and all four `relationship_body` branches.

**BNF rules:** `<element_pattern_where_clause>`, `<element_pattern_predicate>`

```cypher
MATCH (n WHERE n.age > 30)-[:KNOWS]->(m WHERE m.name = 'Alice')
MATCH ()-[r WHERE r.weight > 5]-()
```

### 6. INF / INFINITY / NAN literals

**Status**: ✅ **RESOLVED** in `008-opencypher-bnf-gap` — `inf_literal`, `infinity_literal`, `nan_literal` rules added; listed in `expression` before `$.identifier`.

**BNF rule:** `<signed_numeric_literal>`

```cypher
RETURN INF, INFINITY, NAN
```

### 7. YIELD … WHERE in CALL

**Status**: ✅ **RESOLVED** in `008-opencypher-bnf-gap` — `optional($.where_clause)` added to `yield_clause`.

**BNF rule:** `<yield_clause>`

```cypher
CALL db.labels() YIELD label WHERE label STARTS WITH 'P'
```

### 8. Relationship label wildcard `%` and `IS` label expression

**Status**: ⬜ **OPEN** — not addressed in this feature.

**BNF rule:** `<wildcard_label>`

```cypher
MATCH (n IS %)-[r IS %]-(m)
MATCH (n WHERE n IS Person&Employee)
```

The `%` wildcard label is part of the spec. Neither `%` in label position nor the `IS` form inside element
patterns is fully supported.

### 9. String concatenation operator `||`

**Status**: ⬜ **OPEN** — not addressed; `||` parses but is not a distinct named node.

**BNF rule:** `<concatenation_operator>`

```cypher
RETURN 'hello' || ' ' || 'world'
```

### 10. Float type suffixes and underscore digit separators

**Status**: ✅ **RESOLVED** in `008-opencypher-bnf-gap` — `float_literal` and `integer_literal` regexes updated to allow `_` separators and `F`/`D` suffixes.

**BNF rules:** `<approximate_number_suffix>`, `<unsigned_decimal_integer>`

```cypher
RETURN 1.5f, 2.0D, 1_000_000
```
