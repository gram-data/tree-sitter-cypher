; Scope boundaries
(statement) @local.scope
(union_statement) @local.scope

; Variable definitions — graph pattern bindings
(node_pattern variable: (identifier) @local.definition)
(relationship_body variable: (identifier) @local.definition)
(path_pattern variable: (identifier) @local.definition)

; Variable definitions — clause-level bindings
(return_item alias: (identifier) @local.definition)
(return_item alias: (escaped_identifier) @local.definition)
(yield_item alias: (identifier) @local.definition)
(unwind_clause (identifier) @local.definition)

; Variable definitions — comprehension and quantifier bindings
(list_comprehension (identifier) @local.definition)
(all_expression (identifier) @local.definition)
(any_expression (identifier) @local.definition)
(none_expression (identifier) @local.definition)
(single_expression (identifier) @local.definition)
(reduce_expression accumulator: (identifier) @local.definition)
(reduce_expression (identifier) @local.definition)

; Generic reference fallback (all other identifier uses)
(identifier) @local.reference
