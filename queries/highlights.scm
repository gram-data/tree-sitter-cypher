; Keywords — clause level
"match" @keyword
"return" @keyword
"create" @keyword
"merge" @keyword
"delete" @keyword
"set" @keyword
"remove" @keyword
"with" @keyword
(starts_with_expression "with" @keyword.operator)
(ends_with_expression "with" @keyword.operator)
"unwind" @keyword
"call" @keyword
"yield" @keyword
"union" @keyword
"where" @keyword
"order" @keyword
"by" @keyword
"skip" @keyword
"offset" @keyword
"limit" @keyword
"on" @keyword
"case" @keyword
"when" @keyword
"then" @keyword
"else" @keyword
"end" @keyword
"as" @keyword

; Keywords — logical and predicate operators
"and" @keyword.operator
"or" @keyword.operator
"xor" @keyword.operator
"not" @keyword.operator
"in" @keyword.operator
"is" @keyword.operator
"contains" @keyword.operator
"starts" @keyword.operator
"ends" @keyword.operator
(union_statement "all" @keyword)
(all_expression "all" @keyword.operator)
"any" @keyword.operator
"none" @keyword.operator
"single" @keyword.operator
"reduce" @keyword.operator

; Keywords — control and modifiers
"optional" @keyword.control
"distinct" @keyword.control
"detach" @keyword.control

"asc" @keyword.modifier
"ascending" @keyword.modifier
"desc" @keyword.modifier
"descending" @keyword.modifier

; Comments
(doc_comment) @comment.documentation
(comment) @comment

; Literals
(string_literal) @string
(integer_literal) @number
(float_literal) @number
(inf_literal) @number
(infinity_literal) @number
(nan_literal) @number
(path_length) @number
(count_star) @function
(boolean_literal) @boolean
(null_literal) @constant.builtin
(is_null_expression "null" @constant.builtin)
(parameter) @variable.parameter

; Labels and relationship types (before generic identifier fallback)
(label_expression label_name: (identifier) @type)

; Function and procedure names (before generic identifier fallback)
(function_name (identifier) @function)
(procedure_name (identifier) @function)

; Property keys (map literal keys and SET/REMOVE property positions)
(property_key_value (identifier) @property)
(property_access property: (identifier) @property)
(property_access property: (escaped_identifier) @property)

; Operators — punctuation brackets
"(" @punctuation.bracket
")" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket

; Operators — punctuation delimiters
"," @punctuation.delimiter
";" @punctuation.delimiter
"." @punctuation.delimiter

; Operators — relational and arithmetic
"->" @operator
"<-" @operator
"=" @operator
"<>" @operator
"<" @operator
">" @operator
"<=" @operator
">=" @operator
"=~" @operator
"+" @operator
"-" @operator
"*" @operator
"/" @operator
"%" @operator
"^" @operator
"+=" @operator
"||" @operator
"!" @operator
"&" @operator
"|" @operator
".." @operator
(binary_expression operator: ["=" "<>" "<" ">" "<=" ">=" "=~" "+" "-" "||" "*" "/" "%" "^"] @operator)

; Quantified path patterns
(fixed_quantifier "{" @punctuation.bracket)
(fixed_quantifier "}" @punctuation.bracket)
(general_quantifier "{" @punctuation.bracket)
(general_quantifier "}" @punctuation.bracket)
(general_quantifier "," @punctuation.delimiter)

; GQL path-search prefix keywords
(all_path_search "all" @keyword)
(any_path_search "any" @keyword)
(all_shortest_path_search "all" @keyword)
(all_shortest_path_search "shortest" @keyword)
(any_shortest_path_search "any" @keyword)
(any_shortest_path_search "shortest" @keyword)
(counted_shortest_path_search "shortest" @keyword)
(counted_shortest_group_search "shortest" @keyword)
(counted_shortest_group_search "groups" @keyword)
(counted_shortest_group_search "group" @keyword)

; Map projection
(field_selector "." @punctuation.delimiter)
(field_selector property: (identifier) @property)
(all_fields_selector) @operator
(literal_map_field key: (identifier) @property)

; Shortest path functions (legacy openCypher shortestPath / allShortestPaths)
"shortestpath" @function.builtin
"allshortestpaths" @function.builtin

; New node types from TCK conformance (004-tck-conformance)
(exists_expression "exists" @keyword)
(exists_expression "{" @punctuation.bracket)
(exists_expression "}" @punctuation.bracket)
(is_labeled_expression label: (label_expression) @type)
(pattern_comprehension "|" @operator)
(pattern_comprehension variable: (identifier) @variable.special)

; Generic identifier fallback (must come after all specific captures)
(identifier) @variable
(escaped_identifier) @variable
