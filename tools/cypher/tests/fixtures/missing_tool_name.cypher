/**
 * @param {string} name - Name to filter by
 */
MATCH (n:Person {name: $name})
RETURN n
