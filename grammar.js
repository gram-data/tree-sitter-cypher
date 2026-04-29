/**
 * @file Graph query language
 * @author Andreas Kollegger <andreas.kollegger@neo4j.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "cypher",

  rules: {
    // TODO: add the actual grammar rules
    source_file: $ => "hello"
  }
});
