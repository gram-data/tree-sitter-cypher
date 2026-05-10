;; Rule: DynamicProperty
;; Severity: Information
;; Applies-to: structural
;; Message: Dynamic property key prevents index use. Consider using a static property name if the key is known at query-write time.
;; Code: 03N95
(subscript_expression
  (expression)
  (expression
    [(parameter) (identifier)] @key))
