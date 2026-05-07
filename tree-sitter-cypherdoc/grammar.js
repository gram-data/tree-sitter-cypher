/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: 'cypherdoc',

  extras: $ => [
    /\s+/,
    /[ \t]*\*[ \t\n]/,
  ],

  rules: {
    document: $ => seq(
      '/**',
      $.name,
      optional($.description),
      repeat(choice($.param_tag, $.returns_tag)),
      '*/',
    ),

    name: _ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    description: $ => repeat1($.description_line),

    description_line: _ => /[^@\n *][^\n]*/,

    param_tag: $ => seq(
      '@param',
      field('type', $.type_annotation),
      field('param', choice($.required_param, $.optional_param)),
      optional(field('description', $.tag_description)),
    ),

    returns_tag: $ => seq(
      '@returns',
      field('type', $.returns_type_annotation),
      optional(field('description', $.tag_description)),
    ),

    required_param: $ => field('name', $.identifier),

    optional_param: $ => seq(
      '[',
      field('name', $.identifier),
      '=',
      field('default', $.param_default),
      ']',
    ),

    param_default: $ => choice(
      $.string_default,
      $.number_default,
      $.boolean_default,
    ),

    string_default: _ => choice(
      /"[^"]*"/,
      /'[^']*'/,
    ),

    number_default: _ => /-?[0-9]+(\.[0-9]+)?/,

    boolean_default: _ => choice('true', 'false'),

    type_annotation: $ => seq(
      '{',
      field('type', $.scalar_type),
      '}',
    ),

    returns_type_annotation: $ => seq(
      '{',
      field('type', $.tuple_type),
      '}',
    ),

    tuple_type: $ => seq(
      '[',
      commaSep1($.tuple_member),
      ']',
      optional($.array_marker),
    ),

    tuple_member: $ => seq(
      field('column', $.identifier),
      ':',
      field('type', $.scalar_type),
    ),

    array_marker: _ => '[]',

    scalar_type: $ => seq(
      field('name', $.identifier),
      optional(seq(
        '<',
        field('argument', $.type_argument),
        '>',
      )),
    ),

    type_argument: $ => field('value', $.identifier),

    tag_description: _ => /-[^\n]*/,

    identifier: _ => /[a-zA-Z_][a-zA-Z0-9_]*/,
  },
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}
