; Keywords — clause level
"match" @keyword
"return" @keyword
"create" @keyword
"merge" @keyword
"delete" @keyword
"set" @keyword
"remove" @keyword
"with" @keyword
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
"all" @keyword.operator
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
(comment) @comment

; Literals
(string_literal) @string
(integer_literal) @number
(float_literal) @number
(path_length) @number
(count_star) @function
(boolean_literal) @boolean
(null_literal) @constant.builtin
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
(binary_expression operator: _ @operator)

; Generic identifier fallback (must come after all specific captures)
(identifier) @variable
(escaped_identifier) @variable
