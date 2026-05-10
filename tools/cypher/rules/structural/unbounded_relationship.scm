;; Rule: UnboundedRelationship
;; Severity: Error
;; Applies-to: structural
;; Message: Variable-length relationships without an upper limit can hang the database. Add a depth limit, e.g., [*..5].
;; Help: Add an upper bound to the path length, e.g., change [*] to [*..5].
(
  (relationship_body
    length: (path_length) @path_length)
  (#not-match? @path_length "\\.\\.[0-9]"))
