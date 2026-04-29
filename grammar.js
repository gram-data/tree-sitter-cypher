/**
 * @file Graph query language
 * @author Andreas Kollegger <andreas.kollegger@neo4j.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

// T005: case-insensitive keyword terminal (BNF: all reserved words)
const kw = str =>
  token(new RegExp(
    str.split('').map(c =>
      /[a-zA-Z]/.test(c) ? `[${c.toUpperCase()}${c.toLowerCase()}]` : c
    ).join('')
  ));

// T006: comma-separated list helpers (BNF: { <item> [ { , <item> }... ] })
const commaSep1 = rule => seq(rule, repeat(seq(',', rule)));
const commaSep  = rule => optional(commaSep1(rule));

export default grammar({
  name: 'cypher',

  extras: $ => [/\s/, $._comment],

  rules: {

    // ─── T019: Program / Statement ───────────────────────────────────────────
    // BNF: <program> ::= <statement block>
    // BNF: <statement block> ::= <statement>
    // Note: multi-statement files use ';' as separator (expanded in US8)
    source_file: $ => seq(
      $.statement,
      repeat(seq(';', $.statement)),
      optional(';'),
    ),

    // BNF: <linear statement> — reading clauses followed by optional result clause,
    //                           or a standalone RETURN (e.g. RETURN 42)
    statement: $ => choice(
      seq(repeat1($.match_clause), optional($.return_clause)),
      $.return_clause,
    ),

    // ─── T020: MATCH clause ───────────────────────────────────────────────────
    // BNF: <simple match statement> ::= MATCH <pattern>
    // BNF: <optional match statement> ::= OPTIONAL MATCH <pattern>
    match_clause: $ => seq(
      optional(kw('OPTIONAL')),
      kw('MATCH'),
      field('pattern', $.pattern),
      optional(field('where', $.where_clause)),
    ),

    // ─── T029: Node pattern (extended from US2 stub) ─────────────────────────
    // BNF: <node pattern> ::= ( [<binding variable>] [<is label expression>] [<properties>] )
    node_pattern: $ => seq(
      '(',
      optional(field('variable', $.identifier)),
      optional(field('label', $.label_expression)),
      optional(field('properties', $.property_map)),
      ')',
    ),

    // ─── T030: Label expression ───────────────────────────────────────────────
    // BNF: <is node label expression> ::= : <label> | IS <label expression>
    // BNF: <label expression> ::= <label term> | <label expression> | <label term>
    label_expression: $ => choice(
      // IS <label expression> (new-style)
      seq(kw('IS'), $._label_expr_inner),
      // :<label expression> (legacy colon-prefixed)
      seq(':', $._label_expr_inner),
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
    path_pattern: $ => seq(
      optional(seq(field('variable', $.identifier), '=')),
      $.node_pattern,
      repeat(seq($.relationship_pattern, $.node_pattern)),
    ),

    // ─── T032: Relationship pattern ───────────────────────────────────────────
    // BNF: <relationship pattern> — directed or undirected, with optional body
    relationship_pattern: $ => choice(
      seq('<-', '[', optional($.relationship_body), ']', '-'),
      seq('-',  '[', optional($.relationship_body), ']', '->'),
      seq('-',  '[', optional($.relationship_body), ']', '-'),
      seq('<-', '-'),
      seq('-',  '->'),
      seq('-',  '-'),
    ),

    // ─── T033: Relationship body ─────────────────────────────────────────────
    // BNF: <relationship detail> ::= [<variable>] [<label>] [<path length>] [<properties>]
    // At least one component must be present (tree-sitter cannot parse empty-matching rules).
    // Usage sites use optional($.relationship_body) to handle the no-body case.
    relationship_body: $ => choice(
      seq(field('variable', $.identifier),
          optional(field('label', $.label_expression)),
          optional(field('length', $.path_length)),
          optional(field('properties', $.property_map))),
      seq(field('label', $.label_expression),
          optional(field('length', $.path_length)),
          optional(field('properties', $.property_map))),
      seq(field('length', $.path_length),
          optional(field('properties', $.property_map))),
      field('properties', $.property_map),
    ),

    // ─── T034: Path length ────────────────────────────────────────────────────
    // BNF: <path length> ::= * | *<n> | *<n>..<m> | *..<m>
    path_length: _ => token(seq(
      '*',
      optional(seq(
        /[0-9]+/,
        optional(seq('..', /[0-9]*/)),
      )),
    )),

    // ─── T022: WHERE clause ───────────────────────────────────────────────────
    // BNF: <where clause> ::= WHERE <value expression>
    where_clause: $ => seq(
      kw('WHERE'),
      $.expression,
    ),

    // ─── T023: RETURN clause ─────────────────────────────────────────────────
    // BNF: <return statement> ::= RETURN <return statement body>
    return_clause: $ => seq(
      kw('RETURN'),
      $.return_body,
    ),

    // BNF: <return statement body> ::= [DISTINCT] <return item list>
    return_body: $ => seq(
      optional(kw('DISTINCT')),
      commaSep1($.return_item),
    ),

    // BNF: <return item> ::= <value expression> [AS <identifier>]
    return_item: $ => seq(
      $.expression,
      optional(seq(kw('AS'), field('alias', $.identifier))),
    ),

    // ─── T024/T025: Expression (placeholder, extended in US4) ────────────────
    // BNF: <value expression> — covers all expression forms
    expression: $ => choice(
      $.binary_expression,
      $.property_access,
      $.integer_literal,
      $.float_literal,
      $.string_literal,
      $.boolean_literal,
      $.null_literal,
      $.parameter,
      $.escaped_identifier,
      $.identifier,
    ),

    // BNF: <boolean value expression>, <arithmetic value expression>, etc.
    binary_expression: $ => prec.left(1, seq(
      field('left', $.expression),
      field('operator', choice('=', '<>', '<', '>', '<=', '>=')),
      field('right', $.expression),
    )),

    // ─── T025: Property access ────────────────────────────────────────────────
    // BNF: <postfix expression> with property name
    property_access: $ => prec.left(10, seq(
      field('object', choice($.identifier, $.escaped_identifier, $.parameter)),
      '.',
      field('property', $.identifier),
    )),

    // ─── T010–T015: Literals and terminals (from US1) ────────────────────────

    integer_literal: _ => token(choice(
      /0[xX][0-9a-fA-F]+/,
      /0[oO][0-7]+/,
      /[0-9]+/,
    )),

    float_literal: _ => token(choice(
      /[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?/,
      /\.[0-9]+([eE][+-]?[0-9]+)?/,
      /[0-9]+[eE][+-]?[0-9]+/,
    )),

    string_literal: _ => token(choice(
      seq('"', /([^"\\]|\\.)*/,  '"'),
      seq("'", /([^'\\]|\\.)*/,  "'"),
    )),

    boolean_literal: _ => token(choice(kw('true'), kw('false'))),
    null_literal:    _ => token(kw('null')),

    identifier: _ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    escaped_identifier: _ => token(seq('`', /[^`]+/, '`')),

    parameter: _ => token(seq(
      '$',
      choice(/[a-zA-Z_][a-zA-Z0-9_]*/, /[0-9]+/),
    )),

    _comment: _ => token(choice(
      seq('//', /.*/),
      seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/'),
    )),
  },
});
