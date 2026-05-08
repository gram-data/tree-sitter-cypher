;; Rule: UnusedParameter
;; Severity: Warning
;; Applies-to: cross-reference
;; Message: Parameter is declared as @param in the doc comment but never used in the query.
(param_tag
  param: [(required_param name: (identifier) @declared_param)
          (optional_param name: (identifier) @declared_param)])
