; Tool name
(document (name) @name)

; Tag keywords
"@param" @tag
"@returns" @tag

; Type names
(scalar_type (identifier) @type)

; Type arguments (e.g. <Person>)
(type_argument (identifier) @type.argument)

; Parameter and column identifiers
(required_param (identifier) @variable)
(optional_param (identifier) @variable)
(tuple_member column: (identifier) @variable)

; Default values
(string_default) @string
(number_default) @number
(boolean_default) @constant.builtin

; Descriptions
(tag_description) @comment
(description_line) @comment

; Punctuation
"[" @punctuation.bracket
"]" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket
"<" @punctuation.bracket
">" @punctuation.bracket
":" @punctuation.delimiter
"," @punctuation.delimiter
"=" @operator
(array_marker) @operator
