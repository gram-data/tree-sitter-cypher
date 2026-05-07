RETURN "hello world" as message
;
/**
 * @param name
 */
MATCH (n) WHERE n.name = $name
RETURN "Hello " + n.name
