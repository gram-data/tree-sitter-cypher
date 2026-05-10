;; Rule: DeprecatedFunction
;; Severity: Warning
;; Applies-to: structural
;; Message: id() is deprecated in Neo4j 5. Use elementId() instead, which returns a stable string identifier.
;; Code: 01N01
(function_call
  name: (function_name
    . (identifier) @fn .)
  (#eq? @fn "id"))
