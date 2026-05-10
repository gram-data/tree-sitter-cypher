;; Rule: CartesianProduct
;; Severity: Warning
;; Applies-to: structural
;; Message: Disconnected MATCH patterns produce a cartesian product. Connect the patterns with a relationship or split into separate MATCH clauses.
;; Code: 03N90
(match_clause
  pattern: (pattern
    (path_pattern)
    (path_pattern) @hit))
