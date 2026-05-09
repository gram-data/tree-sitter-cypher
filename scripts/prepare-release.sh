#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-}"

if [[ -z "$VERSION" ]]; then
  echo "usage: scripts/prepare-release.sh <version>" >&2
  echo "example: scripts/prepare-release.sh 0.2.0" >&2
  exit 1
fi

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
  echo "error: version must be semver (e.g. 0.2.0 or 0.2.0-rc.1)" >&2
  exit 1
fi

if ! command -v tree-sitter &>/dev/null; then
  echo "error: tree-sitter CLI not found — npm install -g tree-sitter-cli" >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "==> Preparing release v$VERSION"

# Patch [workspace.package] version in Cargo.toml.
# tree-sitter version handles [package] version and package.json, but not the
# workspace table — which cypher-data inherits via version.workspace = true.
sed -i.bak "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Update main grammar: package.json + [package] version in Cargo.toml
echo "    tree-sitter-cypher"
tree-sitter version "$VERSION"

# Update cypherdoc sub-grammar
echo "    tree-sitter-cypherdoc"
(cd tree-sitter-cypherdoc && tree-sitter version "$VERSION")

# Patch version constraints in cypher-data's path dependencies.
# These must match the published crate versions; tree-sitter version doesn't touch them.
sed -i.bak "s/\(tree-sitter-cypher = {[^}]*version = \"\)[^\"]*\"/\1$VERSION\"/" tools/cypher/Cargo.toml
sed -i.bak "s/\(tree-sitter-cypherdoc = {[^}]*version = \"\)[^\"]*\"/\1$VERSION\"/" tools/cypher/Cargo.toml
rm tools/cypher/Cargo.toml.bak

echo ""
echo "Updated files:"
echo "  Cargo.toml                          ([workspace.package] + [package])"
echo "  package.json"
echo "  tree-sitter-cypherdoc/Cargo.toml"
echo "  tree-sitter-cypherdoc/package.json"
echo "  tools/cypher/Cargo.toml             (dependency version constraints)"
echo ""
echo "Next steps:"
echo "  git diff                            # verify changes"
echo "  git add Cargo.toml Cargo.lock package.json tree-sitter-cypherdoc/Cargo.toml tree-sitter-cypherdoc/package.json tools/cypher/Cargo.toml"
echo "  git commit -m 'chore: release v$VERSION'"
echo "  git tag v$VERSION && git push origin main --tags"
