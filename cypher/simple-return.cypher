RETURN "hello world" AS message
;
/**
 * greet_by_name
 *
 * Find a person by name and return a greeting.
 *
 * @param {string} name - The name to search for
 * @returns {[greeting: string]} - The greeting message
 */
MATCH (n) WHERE n.name = $name
RETURN "Hello " + n.name AS greeting
