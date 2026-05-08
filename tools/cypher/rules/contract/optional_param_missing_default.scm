;; Rule: OptionalParamMissingDefault
;; Severity: Error
;; Applies-to: contract
;; Message: Optional @param must declare a default value, e.g., [name="default"].
;; A bare [name] without = default fails to parse as optional_param; tree-sitter
;; produces an ERROR node at the document level rather than inside param_tag.
(document
  (ERROR) @malformed_param)
