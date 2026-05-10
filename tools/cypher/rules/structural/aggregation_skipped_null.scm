;; Rule: AggregationSkippedNull
;; Severity: Information
;; Applies-to: structural
;; Message: Aggregation functions silently skip null values, which may give unexpected results if nulls are present.
;; Help: Filter nulls explicitly before aggregating, e.g., add WHERE n.prop IS NOT NULL, or use coalesce(n.prop, 0) inside the function.
(function_call
  name: (function_name
    . (identifier) @fn .)
  (#match? @fn "(?i)^(sum|avg|min|max|collect|stdev|stdevp|percentilecont|percentiledisc)$"))
