/**
 * find_nodes
 * @param {string} name - Name to filter by
 * @param {string} [label] - Bare optional param with no default
 */
MATCH (n:Person {name: $name})
RETURN n
