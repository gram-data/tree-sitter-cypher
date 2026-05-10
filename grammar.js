/**
 * @file Graph query language
 * @author Andreas Kollegger <andreas.kollegger@neo4j.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

// T003: case-insensitive keyword terminal producing a capturable anonymous node.
// Each kw('MATCH') call emits alias(token_(...), 'match') so query files can
// capture keywords as '"match" @keyword' without naming every keyword as a rule.
const kw = str => {
  const t = token(new RegExp(
    str.split('').map(c =>
      /[a-zA-Z]/.test(c) ? `[${c.toUpperCase()}${c.toLowerCase()}]` : c
    ).join('')
  ));
  return alias(t, str.toLowerCase());
};

// T006: comma-separated list helpers (BNF: { <item> [ { , <item> }... ] })
const commaSep1 = rule => seq(rule, repeat(seq(',', rule)));
const commaSep  = rule => optional(commaSep1(rule));

export default grammar({
  name: 'cypher',

  extras: $ => [/\s/, $.comment, $.doc_comment],

  // GLR disambiguation for ambiguous token sequences
  conflicts: $ => [
    [$.expression, $.function_name],           // identifier '.' → property_access vs qualified function_name
    [$.set_item, $.expression],                // SET identifier : → label set vs expression
    [$.remove_item, $.expression],             // REMOVE identifier : → label remove vs expression
    [$.is_labeled_expression, $.set_item],     // n:Label → label predicate vs SET label
    [$.is_labeled_expression, $.remove_item],  // n:Label → label predicate vs REMOVE label
    [$.label_expression],                      // :A:B repeat — consume maximally
    [$.expression, $.pattern_comprehension],   // [ identifier '=' → expression vs path variable
    [$.node_pattern, $.expression],            // [ (identifier) → node_pattern vs (expr)
    [$.property_map, $.map_literal],           // [ ({ }) ] → node property vs map literal in expr
    [$.map_projection, $.map_literal],         // expr { } → map projection vs standalone map literal
    [$.statement],                             // exists_subquery repeat1(statement) — consume greedily
    [$.pattern_predicate],                     // pattern_predicate repeat1 — consume path greedily
    [$.legacy_shortest_path_pattern, $.function_call], // shortestPath( → path pattern vs function call
  ],

  rules: {

    // ─── T073/T074: Program — statement or UNION ─────────────────────────────
    // BNF: <program> ::= <procedure specification> | <standalone procedure call>
    // BNF: <composite statement> ::= <linear statement> [UNION [ALL] <linear statement>]
    source_file: $ => seq(
      choice($.union_statement, $.statement),
      repeat(seq(';', choice($.union_statement, $.statement))),
      optional(';'),
    ),

    // BNF: <composite conjunction> ::= UNION [ALL]
    // Chained: A UNION B UNION C parsed left-recursively
    union_statement: $ => prec.left(0, seq(
      choice($.union_statement, $.statement),
      kw('UNION'),
      optional(kw('ALL')),
      $.statement,
    )),

    // BNF: <linear statement> — any sequence of reading/pipeline/mutation clauses + RETURN
    // A flat clause list avoids ordering conflicts while covering all TCK patterns.
    _updating_clause: $ => choice(
      $.create_clause, $.set_clause, $.remove_clause, $.delete_clause, $.merge_clause,
    ),

    _pipeline_clause: $ => choice(
      $.match_clause, $.with_clause, $.unwind_clause,
      $._updating_clause, $.call_clause,
    ),

    statement: $ => choice(
      seq(repeat1($._pipeline_clause), $.return_clause),
      repeat1($._pipeline_clause),
      $.return_clause,
    ),

    // ─── T020: MATCH clause ───────────────────────────────────────────────────
    // BNF: <simple match statement> ::= MATCH <pattern>
    // BNF: <optional match statement> ::= OPTIONAL MATCH <pattern>
    match_clause: $ => seq(
      optional(kw('OPTIONAL')),
      kw('MATCH'),
      optional($.path_search_prefix),
      field('pattern', $.pattern),
      optional(field('where', $.where_clause)),
    ),

    // BNF: <path search prefix> — GQL-aligned path search modes
    path_search_prefix: $ => choice(
      $.all_path_search,
      $.any_path_search,
      $.all_shortest_path_search,
      $.any_shortest_path_search,
      $.counted_shortest_path_search,
      $.counted_shortest_group_search,
    ),

    // BNF: <all path search> ::= ALL [PATH|PATHS]
    all_path_search: $ => seq(kw('ALL'), optional(choice(kw('PATH'), kw('PATHS')))),

    // BNF: <any path search> ::= ANY [n] [PATH|PATHS]
    any_path_search: $ => seq(
      kw('ANY'),
      optional($.integer_literal),
      optional(choice(kw('PATH'), kw('PATHS'))),
    ),

    // BNF: <all shortest path search> ::= ALL SHORTEST [PATH|PATHS]
    all_shortest_path_search: $ => seq(
      kw('ALL'), kw('SHORTEST'), optional(choice(kw('PATH'), kw('PATHS'))),
    ),

    // BNF: <any shortest path search> ::= ANY SHORTEST [PATH|PATHS]
    any_shortest_path_search: $ => seq(
      kw('ANY'), kw('SHORTEST'), optional(choice(kw('PATH'), kw('PATHS'))),
    ),

    // BNF: <counted shortest path search> ::= SHORTEST n [PATH|PATHS]
    counted_shortest_path_search: $ => seq(
      kw('SHORTEST'),
      $.integer_literal,
      optional(choice(kw('PATH'), kw('PATHS'))),
    ),

    // BNF: <counted shortest group search> ::= SHORTEST [n] [PATH|PATHS] GROUP|GROUPS
    counted_shortest_group_search: $ => seq(
      kw('SHORTEST'),
      optional($.integer_literal),
      optional(choice(kw('PATH'), kw('PATHS'))),
      choice(kw('GROUP'), kw('GROUPS')),
    ),

    // ─── T029: Node pattern (extended from US2 stub) ─────────────────────────
    // BNF: <node pattern> ::= ( [<binding variable>] [<is label expression>] [<properties>] )
    // properties: property_map {k:v} or parameter $p (Neo4j extension for node property matching)
    node_pattern: $ => seq(
      '(',
      optional(field('variable', $.identifier)),
      optional(field('label', $.label_expression)),
      optional(field('properties', choice($.parameter, $.property_map))),
      optional(field('where', $.where_clause)),
      ')',
    ),

    // ─── T030: Label expression ───────────────────────────────────────────────
    // BNF: <is node label expression> ::= : <label> | IS <label expression>
    // BNF: <label expression> ::= <label term> | <label expression> | <label term>
    // Also supports legacy colon-chain: :A:B:C (multiple labels on same node/rel)
    label_expression: $ => choice(
      // IS <label expression> (new-style: IS A&B)
      seq(kw('IS'), $._label_expr_inner),
      // :<label>+ (legacy colon-prefixed, possibly chained)
      seq(':', $._label_expr_inner, repeat(seq(':', $._label_expr_inner))),
    ),

    _label_expr_inner: $ => choice(
      prec.left(3, seq($._label_expr_inner, '&', $._label_expr_inner)),
      prec.left(2, seq($._label_expr_inner, '|', $._label_expr_inner)),
      prec.right(4, seq('!', $._label_expr_inner)),
      seq('(', $._label_expr_inner, ')'),
      // BNF: <label name> — a plain identifier; reuse $.identifier to avoid token conflict
      field('label_name', $.identifier),
    ),

    // ─── T031: Property map ───────────────────────────────────────────────────
    // BNF: <properties> ::= <map literal>
    property_map: $ => seq(
      '{',
      commaSep($.property_key_value),
      '}',
    ),

    property_key_value: $ => seq(
      $.identifier,
      ':',
      $.expression,
    ),

    // ─── T035: Pattern / path pattern ────────────────────────────────────────
    // BNF: <pattern> ::= <path pattern> [ { , <path pattern> }... ]
    pattern: $ => commaSep1($.path_pattern),

    // BNF: <path pattern> ::= [<path variable> =] <path pattern expression>
    // BNF: <path pattern expression> ::= <path term> | <legacy shortest path pattern>
    path_pattern: $ => seq(
      optional(seq(field('variable', $.identifier), '=')),
      choice(
        $.legacy_shortest_path_pattern,
        seq(
          choice($.quantified_path_primary, $.node_pattern),
          repeat(seq($.relationship_pattern, choice($.quantified_path_primary, $.node_pattern))),
        ),
      ),
    ),

    // BNF: <quantified path primary> ::= '(' <node pattern> <relationship pattern> <node pattern>... ')' <quantifier>
    quantified_path_primary: $ => seq(
      '(',
      $.node_pattern,
      repeat1(seq($.relationship_pattern, $.node_pattern)),
      ')',
      field('quantifier', $.graph_pattern_quantifier),
    ),

    // BNF: <graph pattern quantifier> ::= '+' | '*' | <fixed quantifier> | <general quantifier>
    graph_pattern_quantifier: $ => choice(
      '+',
      '*',
      $.fixed_quantifier,
      $.general_quantifier,
    ),

    // BNF: <fixed quantifier> ::= '{' n '}'
    fixed_quantifier: $ => seq(
      '{',
      field('count', $.integer_literal),
      '}',
    ),

    // BNF: <general quantifier> ::= '{' [lower] ',' [upper] '}'
    general_quantifier: $ => seq(
      '{',
      field('lower', optional($.integer_literal)),
      ',',
      field('upper', optional($.integer_literal)),
      '}',
    ),

    // ─── T032: Relationship pattern ───────────────────────────────────────────
    // BNF: <relationship pattern> — directed or undirected, with optional body
    // <-> bidirectional is a Neo4j extension (both-direction explicit markers)
    relationship_pattern: $ => choice(
      seq('<-', '[', optional($.relationship_body), ']', '->'),  // <-[r]-> bidir
      seq('<-', '[', optional($.relationship_body), ']', '-'),   // <-[r]- left
      seq('-',  '[', optional($.relationship_body), ']', '->'),  // -[r]-> right
      seq('-',  '[', optional($.relationship_body), ']', '-'),   // -[r]- undirected
      seq('<-', '->'),   // <--> bidir no body
      seq('<-', '-'),    // <-- left no body
      seq('-',  '->'),   // --> right no body
      seq('-',  '-'),    // -- undirected no body
    ),

    // ─── T033: Relationship body ─────────────────────────────────────────────
    // BNF: <relationship detail> ::= [<variable>] [<label>] [<path length>] [<properties>]
    // properties: property_map {k:v} or parameter $p (Neo4j extension)
    // At least one component must be present (tree-sitter cannot parse empty-matching rules).
    // Usage sites use optional($.relationship_body) to handle the no-body case.
    _rel_props: $ => field('properties', choice($.parameter, $.property_map)),

    relationship_body: $ => choice(
      seq(field('variable', $.identifier),
          optional(field('label', $.label_expression)),
          optional(field('length', $.path_length)),
          optional($._rel_props),
          optional(field('where', $.where_clause))),
      seq(field('label', $.label_expression),
          optional(field('length', $.path_length)),
          optional($._rel_props),
          optional(field('where', $.where_clause))),
      seq(field('length', $.path_length),
          optional($._rel_props),
          optional(field('where', $.where_clause))),
      seq($._rel_props,
          optional(field('where', $.where_clause))),
    ),

    // ─── T034: Path length ────────────────────────────────────────────────────
    // BNF: <path length> ::= * | *<n> | *<n>..<m> | *<n>.. | *..<m> | *..
    path_length: _ => token(seq(
      '*',
      optional(choice(
        seq(/[0-9]+/, '..', /[0-9]*/),  // *N..M or *N..
        seq('..', /[0-9]*/),            // *..M or *..
        /[0-9]+/,                       // *N (exact)
      )),
    )),

    // ─── T057: WITH clause ───────────────────────────────────────────────────
    // BNF: <with statement> ::= WITH <return statement body> [ORDER BY] [SKIP] [LIMIT] [WHERE]
    with_clause: $ => seq(
      kw('WITH'),
      $.return_body,
      optional($.order_by_clause),
      optional($.skip_clause),
      optional($.limit_clause),
      optional($.where_clause),
    ),

    // ─── T058: UNWIND clause ─────────────────────────────────────────────────
    // BNF: <unwind statement> ::= UNWIND <value expression> AS <binding variable>
    unwind_clause: $ => seq(
      kw('UNWIND'),
      $.expression,
      kw('AS'),
      field('variable', $._symbolic_name),
    ),

    // ─── T059: ORDER BY clause ───────────────────────────────────────────────
    // BNF: <order by clause> ::= ORDER BY <sort item list>
    order_by_clause: $ => seq(
      kw('ORDER'), kw('BY'),
      commaSep1($.sort_item),
    ),

    // BNF: <sort item> ::= <value expression> [ASC | DESC | ASCENDING | DESCENDING]
    sort_item: $ => seq(
      $.expression,
      optional(choice(kw('ASC'), kw('DESC'), kw('ASCENDING'), kw('DESCENDING'))),
    ),

    // ─── T060: SKIP and LIMIT ────────────────────────────────────────────────
    // BNF: <skip clause>, <limit clause>
    skip_clause:  $ => seq(choice(kw('SKIP'), kw('OFFSET')), $.expression),
    limit_clause: $ => seq(kw('LIMIT'), $.expression),

    // ─── T064: MERGE clause ──────────────────────────────────────────────────
    // BNF: <merge statement> ::= MERGE <pattern> [<merge action>...]
    merge_clause: $ => seq(
      kw('MERGE'),
      $.pattern,
      repeat($.merge_action),
    ),

    // BNF: <merge action> ::= ON { MATCH | CREATE } <set statement>
    merge_action: $ => seq(
      kw('ON'),
      choice(kw('MATCH'), kw('CREATE')),
      $.set_clause,
    ),

    // ─── T066: CALL clause ────────────────────────────────────────────────────
    // BNF: <named procedure call> / <standalone procedure call>
    // Parentheses are optional for standalone calls (BNF: <standalone procedure call>)
    call_clause: $ => seq(
      kw('CALL'),
      field('name', $.procedure_name),
      optional(seq('(', commaSep($.expression), ')')),
      optional($.yield_clause),
    ),

    // BNF: <procedure reference> — dot-qualified name
    procedure_name: $ => seq(
      $.identifier,
      repeat(seq('.', $.identifier)),
    ),

    // ─── T068: YIELD clause ───────────────────────────────────────────────────
    // BNF: <yield clause> ::= YIELD (<yield item list> | *) [WHERE <expr>]
    yield_clause: $ => seq(
      kw('YIELD'),
      choice('*', commaSep1($.yield_item)),
      optional($.where_clause),
    ),

    // BNF: <yield item> ::= <field name> [AS <identifier>]
    yield_item: $ => seq(
      field('name', $.identifier),
      optional(seq(kw('AS'), field('alias', $.identifier))),
    ),

    // ─── T049: CREATE clause ─────────────────────────────────────────────────
    // BNF: <create statement> ::= CREATE <pattern>
    create_clause: $ => seq(kw('CREATE'), $.pattern),

    // ─── T050: SET clause ────────────────────────────────────────────────────
    // BNF: <set statement> ::= SET <set item list>
    set_clause: $ => seq(kw('SET'), commaSep1($.set_item)),

    // BNF: <set item> — property assignment, map merge, map replace, or label set
    set_item: $ => choice(
      // n.prop = expr  (property assignment)
      seq($.property_access, '=', $.expression),
      // n = expr  (full replace)
      seq($.identifier, '=', $.expression),
      // n += expr  (map merge)
      seq($.identifier, '+=', $.expression),
      // n:Label  (label set)
      seq($.identifier, $.label_expression),
    ),

    // ─── T051: REMOVE clause ─────────────────────────────────────────────────
    // BNF: <remove statement> ::= REMOVE <remove item list>
    remove_clause: $ => seq(kw('REMOVE'), commaSep1($.remove_item)),

    // BNF: <remove item> — label removal or property removal
    remove_item: $ => choice(
      seq($.identifier, $.label_expression),   // n:Label
      $.property_access,                        // n.prop
    ),

    // ─── T052: DELETE clause ─────────────────────────────────────────────────
    // BNF: <delete statement> ::= [DETACH] DELETE <delete item list>
    delete_clause: $ => seq(
      optional(kw('DETACH')),
      kw('DELETE'),
      commaSep1($.expression),
    ),

    // ─── T022: WHERE clause ───────────────────────────────────────────────────
    // BNF: <where clause> ::= WHERE <value expression>
    where_clause: $ => seq(
      kw('WHERE'),
      $.expression,
    ),

    // ─── T023: RETURN clause ─────────────────────────────────────────────────
    // BNF: <return statement> ::= RETURN <return statement body> [ORDER BY] [SKIP] [LIMIT]
    return_clause: $ => seq(
      kw('RETURN'),
      $.return_body,
      optional($.order_by_clause),
      optional($.skip_clause),
      optional($.limit_clause),
    ),

    // BNF: <return statement body> ::= [DISTINCT] <return item list> | *
    return_body: $ => seq(
      optional(kw('DISTINCT')),
      choice('*', commaSep1($.return_item)),
    ),

    // BNF: <return item> ::= <value expression> [AS <identifier>]
    return_item: $ => seq(
      $.expression,
      optional(seq(kw('AS'), field('alias', $._symbolic_name))),
    ),

    // BNF: <symbolic name> — identifier or escaped identifier
    _symbolic_name: $ => choice($.identifier, $.escaped_identifier),

    // ─── T039–T046 + T075–T080: Full expression grammar ─────────────────────
    // BNF: <value expression> → <boolean value expression> → ... → <postfix expression>
    expression: $ => choice(
      // Parenthesized expression (BNF: <parenthesized value expression>)
      seq('(', $.expression, ')'),
      $.binary_expression,
      $.unary_expression,
      $.is_null_expression,
      $.is_labeled_expression,
      $.exists_expression,
      $.pattern_predicate,
      $.in_expression,
      $.starts_with_expression,
      $.ends_with_expression,
      $.contains_expression,
      $.case_expression,
      $.list_comprehension,
      $.pattern_comprehension,
      $.all_expression,
      $.any_expression,
      $.none_expression,
      $.single_expression,
      $.reduce_expression,
      $.count_star,
      $.legacy_shortest_path_pattern,
      $.function_call,
      $.map_projection,
      $.subscript_expression,
      $.property_access,
      $.list_literal,
      $.map_literal,
      $.integer_literal,
      $.float_literal,
      $.string_literal,
      $.boolean_literal,
      $.null_literal,
      $.parameter,
      $.escaped_identifier,
      $.infinity_literal,
      $.inf_literal,
      $.nan_literal,
      $.identifier,
    ),

    // T039: Binary operators with precedence levels matching BNF nesting depth
    // BNF: <boolean value expression>, <arithmetic value expression>, <arithmetic term>
    binary_expression: $ => {
      const table = [
        [prec.left,  1,  kw('OR')],
        [prec.left,  2,  kw('XOR')],
        [prec.left,  3,  kw('AND')],
        [prec.left,  5,  choice('=', '<>', '<', '>', '<=', '>=', '=~')],
        [prec.left,  6,  choice('+', '-', '||')],
        [prec.left,  7,  choice('*', '/', '%')],
        [prec.right, 8,  '^'],
      ];
      return choice(...table.map(([fn, prec_, op]) =>
        fn(prec_, seq(
          field('left', $.expression),
          field('operator', op),
          field('right', $.expression),
        ))
      ));
    },

    // T040: Unary operators — NOT (level 4), unary +/- (level 9)
    // BNF: <boolean factor>, <unary arithmetic>
    unary_expression: $ => choice(
      prec.right(4, seq(kw('NOT'), $.expression)),
      prec.right(9, seq(choice('+', '-'), $.expression)),
    ),

    // T041: IS NULL / IS NOT NULL predicates
    // BNF: <null predicate>
    is_null_expression: $ => prec.left(5, seq(
      $.expression,
      kw('IS'),
      optional(kw('NOT')),
      kw('NULL'),
    )),

    // T041: IN list membership
    // BNF: <in predicate>
    in_expression: $ => prec.left(5, seq(
      $.expression,
      kw('IN'),
      $.expression,
    )),

    // T042: String predicates
    // BNF: <string operator expression>
    starts_with_expression: $ => prec.left(5, seq(
      $.expression, kw('STARTS'), kw('WITH'), $.expression,
    )),
    ends_with_expression: $ => prec.left(5, seq(
      $.expression, kw('ENDS'), kw('WITH'), $.expression,
    )),
    contains_expression: $ => prec.left(5, seq(
      $.expression, kw('CONTAINS'), $.expression,
    )),

    // BNF: <legacy shortest path pattern>
    // shortestPath((a)-[*]-(b)) or allShortestPaths((a)-[:T*]-(b))
    legacy_shortest_path_pattern: $ => seq(
      field('function', choice(kw('SHORTESTPATH'), kw('ALLSHORTESTPATHS'))),
      '(',
      field('start', $.node_pattern),
      $.relationship_pattern,
      field('end', $.node_pattern),
      ')',
    ),

    // T043: Function calls — simple and qualified names (e.g., db.labels)
    // BNF: <function invocation>
    function_call: $ => seq(
      field('name', $.function_name),
      '(',
      optional(kw('DISTINCT')),
      commaSep($.expression),
      ')',
    ),

    // BNF: <function name> — dot-qualified identifier path (syntactic rule, not a token,
    // so disambiguation with property_access happens via '(' lookahead at parser level)
    function_name: $ => seq(
      $.identifier,
      repeat(seq('.', $.identifier)),
    ),

    // T044: List literal and map literal
    // BNF: <list literal>
    list_literal: $ => seq('[', commaSep($.expression), ']'),

    // BNF: <map literal> — same syntax as property_map but used as an expression value
    map_literal: $ => seq('{', commaSep($.property_key_value), '}'),

    // BNF: <map projection> ::= <expression> '{' [<map projection element list>] '}'
    // e.g., n { .name, .age, score: 10, .* }
    map_projection: $ => prec.left(10, seq(
      field('object', $.expression),
      '{',
      commaSep($.map_projection_element),
      '}',
    )),

    // BNF: <map projection element> — one of four forms
    map_projection_element: $ => choice(
      $.all_fields_selector,   // .*
      $.field_selector,        // .name
      $.literal_map_field,     // key: expr
      $.variable_selector,     // bareVar
    ),

    // BNF: <all fields selector> ::= '.' '*'
    all_fields_selector: _ => seq('.', '*'),

    // BNF: <field selector> ::= '.' <property name>
    field_selector: $ => seq('.', field('property', $._symbolic_name)),

    // BNF: <literal map field> ::= <field name> ':' <expression>
    literal_map_field: $ => seq(
      field('key', $.identifier),
      ':',
      field('value', $.expression),
    ),

    // BNF: <variable selector> ::= <variable>
    variable_selector: $ => field('variable', $.identifier),

    // T045: Subscript / index access and slice notation
    // BNF: <subscript operator>, <list slice>
    subscript_expression: $ => prec.left(10, seq(
      $.expression,
      '[',
      choice(
        seq($.expression, '..', optional($.expression)),  // list[1..3] or list[1..]
        seq('..', $.expression),                          // list[..3]
        $.expression,                                     // list[i]
      ),
      ']',
    )),

    // T025: Property access (level 10 postfix, left-recursive for chaining: a.b.c)
    // BNF: <postfix expression> with property name
    property_access: $ => prec.left(10, seq(
      field('object', $.expression),
      '.',
      field('property', $._symbolic_name),
    )),

    // ─── T075: CASE expression ───────────────────────────────────────────────
    // BNF: <case expression> — simple (with operand) or searched (without)
    case_expression: $ => seq(
      kw('CASE'),
      optional($.expression),            // present = simple CASE, absent = searched CASE
      repeat1($.case_when_clause),
      optional($.case_else_clause),
      kw('END'),
    ),

    case_when_clause: $ => seq(
      kw('WHEN'), $.expression,
      kw('THEN'), $.expression,
    ),

    case_else_clause: $ => seq(kw('ELSE'), $.expression),

    // ─── pattern_predicate: path pattern as boolean expression ───────────────
    // BNF: <pattern expression> ::= <simple path pattern> (as <boolean primary>)
    // Handles: WHERE (n)-->(m), WHERE (n)-[:T]-(), WHERE NOT (n)-->(m)
    // Requires at least one relationship — disambiguates from parenthesized expression
    pattern_predicate: $ => seq(
      $.node_pattern,
      repeat1(seq($.relationship_pattern, $.node_pattern)),
    ),

    // ─── exists_expression: EXISTS { } subquery predicate ────────────────────
    // BNF: <exists expression> ::= EXISTS { <subquery expression argument> }
    // <subquery expression argument> ::= <procedure specification> | <graph pattern>
    // <graph pattern> includes optional <graph pattern where clause> (WHERE expr)
    exists_expression: $ => seq(
      kw('EXISTS'),
      '{',
      choice(
        seq($.pattern, optional($.where_clause)),  // graph pattern form (with optional WHERE)
        $.exists_subquery,                          // multi-clause form (MATCH ... RETURN ...)
      ),
      '}',
    ),

    // BNF: <procedure specification> ::= <statement block> (multi-clause subquery)
    exists_subquery: $ => repeat1($.statement),

    // ─── is_labeled_expression: label predicate in boolean context ────────────
    // BNF: <is labeled predicate part 2> ::= <is label expression>
    // Handles: WHERE n:Person, WHERE n IS Person, RETURN n:Foo AS result
    is_labeled_expression: $ => prec.left(5, seq(
      $.expression,
      field('label', $.label_expression),
    )),

    // ─── pattern_comprehension: path pattern as list comprehension ─────────────
    // BNF: <pattern comprehension> ::= '[' <pattern source> <pattern filter and projection> ']'
    // Handles: [(n)-->() | n.name], [p = (n)-->() | p]
    // Requires at least one relationship (disambiguates from list_comprehension / parenthesized expr)
    pattern_comprehension: $ => seq(
      '[',
      optional(seq(field('variable', $.identifier), '=')),
      field('start', $.node_pattern),
      repeat1(seq($.relationship_pattern, $.node_pattern)),
      optional($.where_clause),
      '|',
      field('projection', $.expression),
      ']',
    ),

    // ─── T076: List comprehension ─────────────────────────────────────────────
    // BNF: <list comprehension> ::= '[' <variable> IN <expr> [WHERE <expr>] ['|' <expr>] ']'
    // prec(2) to resolve conflict with in_expression inside list_literal
    list_comprehension: $ => prec(2, seq(
      '[',
      field('variable', $.identifier),
      kw('IN'),
      $.expression,
      optional($.where_clause),
      optional(seq('|', $.expression)),
      ']',
    )),

    // ─── T079: Existential quantifiers ───────────────────────────────────────
    // BNF: <all predicate>, <any predicate>, <none predicate>, <single predicate>
    all_expression:    $ => seq(kw('ALL'),    '(', field('variable', $.identifier), kw('IN'), $.expression, optional($.where_clause), ')'),
    any_expression:    $ => seq(kw('ANY'),    '(', field('variable', $.identifier), kw('IN'), $.expression, optional($.where_clause), ')'),
    none_expression:   $ => seq(kw('NONE'),   '(', field('variable', $.identifier), kw('IN'), $.expression, optional($.where_clause), ')'),
    single_expression: $ => seq(kw('SINGLE'), '(', field('variable', $.identifier), kw('IN'), $.expression, optional($.where_clause), ')'),

    // ─── T078: REDUCE expression ──────────────────────────────────────────────
    // BNF: <reduce expression> ::= REDUCE ( acc = init, var IN list | expr )
    reduce_expression: $ => seq(
      kw('REDUCE'),
      '(',
      field('accumulator', $.identifier),
      '=',
      $.expression,
      ',',
      field('iterator', $.identifier),
      kw('IN'),
      $.expression,
      '|',
      $.expression,
      ')',
    ),

    // ─── T080: count(*) ───────────────────────────────────────────────────────
    // BNF: <count star>
    count_star: _ => token(seq(
      /[Cc][Oo][Uu][Nn][Tt]/,
      /\s*/,
      '(',
      /\s*/,
      '*',
      /\s*/,
      ')',
    )),

    // ─── T010–T015: Literals and terminals (from US1) ────────────────────────

    // BNF: <unsigned decimal integer> — allows underscore digit separators (e.g. 1_000_000)
    integer_literal: _ => token(choice(
      /0[xX][0-9a-fA-F][0-9a-fA-F_]*/,
      /0[oO][0-7][0-7_]*/,
      /[0-9][0-9_]*/,
    )),

    // BNF: <approximate numeric literal> — allows underscore separators and F/D type suffixes
    float_literal: _ => token(choice(
      /[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9][0-9_]*)?[fFdD]?/,
      /\.[0-9][0-9_]*([eE][+-]?[0-9][0-9_]*)?[fFdD]?/,
      /[0-9][0-9_]*[eE][+-]?[0-9][0-9_]*[fFdD]?/,
    )),

    // BNF: <signed numeric literal> — INF, INFINITY, NAN special numeric values
    inf_literal:      _ => token(/[Ii][Nn][Ff]/),
    infinity_literal: _ => token(/[Ii][Nn][Ff][Ii][Nn][Ii][Tt][Yy]/),
    nan_literal:      _ => token(/[Nn][Aa][Nn]/),

    string_literal: _ => token(choice(
      seq('"', /([^"\\]|\\.)*/,  '"'),
      seq("'", /([^'\\]|\\.)*/,  "'"),
    )),

    boolean_literal: _ => token(choice(/[Tt][Rr][Uu][Ee]/, /[Ff][Aa][Ll][Ss][Ee]/)),
    null_literal:    _ => token(/[Nn][Uu][Ll][Ll]/),

    identifier: _ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    escaped_identifier: _ => token(seq('`', /[^`]+/, '`')),

    parameter: _ => token(seq(
      '$',
      choice(/[a-zA-Z_][a-zA-Z0-9_]*/, /[0-9]+/),
    )),

    doc_comment: _ => token(seq('/**', /[^*]*\*+([^/*][^*]*\*+)*/, '/')),

    comment: _ => token(choice(
      seq('//', /.*/),
      seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/'),
    )),
  },
});
