;; Rule: CartesianProduct
;; Severity: Warning
;; Applies-to: structural
;; Message: Multiple comma-separated MATCH patterns may produce a cartesian product if they share no variables. This is a conservative heuristic — connect patterns with a relationship or split into separate MATCH clauses to be explicit.
;; Code: 03N90
;; Help: Connect the patterns with a relationship, e.g., MATCH (a)-[:REL]->(b), or split into separate MATCH clauses.
(match_clause
  pattern: (pattern
    (path_pattern)
    (path_pattern) @hit))
