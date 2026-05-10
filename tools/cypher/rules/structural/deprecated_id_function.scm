;; Rule: DeprecatedFunction
;; Severity: Warning
;; Applies-to: structural
;; Message: id() is deprecated in Neo4j 5. Use elementId() instead, which returns a stable string identifier.
;; Code: 01N01
;; Help: Replace id(n) with elementId(n). Note that elementId() returns a String, not an integer.
(function_call
  name: (function_name
    . (identifier) @fn .)
  (#eq? @fn "id"))
