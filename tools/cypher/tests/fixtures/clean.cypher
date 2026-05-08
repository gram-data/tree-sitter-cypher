/**
 * find_person
 * @param {string} name - Person's name to search for
 * @param {integer} [limit=25] - Maximum number of results
 * @returns {[{person: node}]} - Matching Person nodes
 */
MATCH (person:Person {name: $name})
RETURN person
LIMIT $limit
