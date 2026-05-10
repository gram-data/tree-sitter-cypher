#!/usr/bin/env bash
# Bump all package versions and prepare a release commit.
#
# Usage:
#   ./scripts/prepare-release.sh --bump patch|minor|major
#   ./scripts/prepare-release.sh <version>
#
# Covers:
#   Cargo.toml ([workspace.package])   — sed (tree-sitter version skips this table)
#   tree-sitter version <ver>          — package.json, Cargo.toml [package]
#   tree-sitter-cypherdoc/             — same, via sub-invocation
#   tools/cypher/Cargo.toml            — dependency version constraints

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# ---------------------------------------------------------------------------
# Prerequisites
# ---------------------------------------------------------------------------
for cmd in tree-sitter node; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "error: '$cmd' not found on PATH" >&2
    exit 1
  fi
done

# ---------------------------------------------------------------------------
# Args
# ---------------------------------------------------------------------------
if [[ $# -eq 0 ]]; then
  echo "Usage: $0 --bump patch|minor|major" >&2
  echo "       $0 <version>" >&2
  exit 1
fi

compute_bump() {
  local current="$1" component="$2"
  local major minor patch
  IFS='.' read -r major minor patch <<< "${current%%-*}"
  case "$component" in
    major) echo "$((major + 1)).0.0" ;;
    minor) echo "$major.$((minor + 1)).0" ;;
    patch) echo "$major.$minor.$((patch + 1))" ;;
    *) echo "error: --bump requires patch, minor, or major" >&2; exit 1 ;;
  esac
}

VERSION=""
if [[ "$1" == "--bump" ]]; then
  if [[ $# -lt 2 ]]; then
    echo "error: --bump requires patch, minor, or major" >&2
    exit 1
  fi
  CURRENT=$(cd "$REPO_ROOT" && node -p "require('./package.json').version")
  VERSION=$(compute_bump "$CURRENT" "$2")
else
  VERSION="$1"
  if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
    echo "error: version must be semver (e.g. 0.2.0 or 0.2.0-rc.1)" >&2
    exit 1
  fi
fi

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
sed -i.bak \
  -e "s/\(tree-sitter-cypher = {[^}]*version = \"\)[^\"]*\"/\1$VERSION\"/" \
  -e "s/\(tree-sitter-cypherdoc = {[^}]*version = \"\)[^\"]*\"/\1$VERSION\"/" \
  tools/cypher/Cargo.toml
rm tools/cypher/Cargo.toml.bak

echo ""
echo "Version $VERSION ready. Changed files:"
git diff --name-only | sed 's/^/  /'
echo ""
echo "Next steps:"
echo ""
echo "  git commit -am \"chore: release v$VERSION\""
echo "  git tag v$VERSION && git push origin main --tags"
