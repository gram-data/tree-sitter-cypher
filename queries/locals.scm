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
(unwind_clause variable: (identifier) @local.definition)
(unwind_clause variable: (escaped_identifier) @local.definition)

; Variable definitions — comprehension and quantifier bindings
(list_comprehension variable: (identifier) @local.definition)
(all_expression variable: (identifier) @local.definition)
(any_expression variable: (identifier) @local.definition)
(none_expression variable: (identifier) @local.definition)
(single_expression variable: (identifier) @local.definition)
(reduce_expression accumulator: (identifier) @local.definition)
(reduce_expression iterator: (identifier) @local.definition)

; Generic reference fallback (all other identifier uses)
(identifier) @local.reference
