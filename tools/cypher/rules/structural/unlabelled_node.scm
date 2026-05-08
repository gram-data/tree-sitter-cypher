;; Rule: UnlabelledNode
;; Severity: Warning
;; Applies-to: structural
;; Message: Node pattern without a label causes a full node scan. Add a label, e.g., (n:Person).
(node_pattern
  variable: (identifier)
  !label) @unlabelled_node
