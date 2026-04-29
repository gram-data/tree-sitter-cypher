; Literals
(integer_literal) @number
(float_literal) @number
(string_literal) @string
(boolean_literal) @boolean
(null_literal) @constant.builtin
(parameter) @variable.parameter

; Identifiers
(identifier) @variable
(escaped_identifier) @variable
(label_expression (identifier) @type)
(function_name (identifier) @function)
(procedure_name (identifier) @function)

; Operators
(binary_expression operator: _ @operator)
