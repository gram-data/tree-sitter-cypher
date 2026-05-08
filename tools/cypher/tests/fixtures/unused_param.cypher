/**
 * find_nodes
 * @param {string} label - Node label (unused in query below)
 * @param {string} name - Name to filter by
 */
MATCH (n:Person {name: $name})
RETURN n
