;; Rule: UnlabelledNode
;; Severity: Warning
;; Applies-to: structural
;; Message: Node pattern without a label causes a full node scan. Add a label, e.g., (n:Person).
;; Help: Add a label to restrict which nodes are scanned, e.g., change (n) to (n:Person).
(node_pattern
  variable: (identifier)
  !label) @unlabelled_node
