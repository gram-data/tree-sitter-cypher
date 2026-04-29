#!/usr/bin/env sh
# Extract Cypher queries from openCypher TCK Gherkin feature files.
# Only extracts "When executing query:" blocks (not control queries or assertions).
# Output: one .cypher file per query in /tmp/tck-queries/

set -e

OUT_DIR="${1:-/tmp/tck-queries}"
TCK_DIR="$(dirname "$0")/../references/openCypher/tck/features"

mkdir -p "$OUT_DIR"
rm -f "$OUT_DIR"/*.cypher

n=0
find "$TCK_DIR" -name "*.feature" | sort | while read -r feature_file; do
  awk -v out_dir="$OUT_DIR" -v n_init="$n" '
    BEGIN { in_query=0; in_block=0; query=""; n=n_init+0 }
    /When executing query:/ { in_query=1; next }
    in_query && /"""/ {
      if (in_block) {
        n++
        fname = out_dir "/" sprintf("%04d", n) ".cypher"
        printf "%s", query > fname
        close(fname)
        query=""
        in_block=0
        in_query=0
      } else {
        in_block=1
      }
      next
    }
    in_query && in_block { query = query $0 "\n"; next }
    { in_query=0; in_block=0 }
  ' "$feature_file"
  n=$(ls "$OUT_DIR"/*.cypher 2>/dev/null | wc -l | tr -d ' ')
done

echo "Extracted $(ls "$OUT_DIR"/*.cypher 2>/dev/null | wc -l | tr -d ' ') Cypher queries to $OUT_DIR"
