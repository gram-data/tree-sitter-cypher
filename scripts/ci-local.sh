#!/usr/bin/env sh
# Local pre-push CI check — mirrors .github/workflows/ci.yml.
# Run from the repository root before opening a PR.
#
# Usage:
#   sh scripts/ci-local.sh          # run all three checks
#   sh scripts/ci-local.sh grammar  # cypher grammar only
#   sh scripts/ci-local.sh doc      # cypherdoc grammar only
#   sh scripts/ci-local.sh tool     # cypher-data CLI only

set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# ── helpers ──────────────────────────────────────────────────────────────────

pass() { printf '\033[32m✓\033[0m %s\n' "$*"; }
fail() { printf '\033[31m✗\033[0m %s\n' "$*" >&2; }

section() {
    printf '\n\033[1;34m══ %s ══\033[0m\n' "$*"
}

check_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        fail "Required command not found: $1"
        exit 1
    fi
}

# ── job 1: Cypher grammar ─────────────────────────────────────────────────────

job_grammar() {
    section "Cypher grammar"
    check_cmd node
    check_cmd npx

    printf 'Installing npm dependencies...\n'
    npm ci --omit=peer --omit=optional --silent

    printf 'Regenerating parser...\n'
    TREE_SITTER_ABI_VERSION=15 tree-sitter generate

    printf 'Running corpus tests...\n'
    tree-sitter test

    if [ -d "node_modules/tree-sitter" ]; then
        printf 'Running Node binding tests...\n'
        npm test --silent
    else
        printf 'Skipping Node binding tests (tree-sitter native addon not available with this Node version)\n'
    fi

    pass "Cypher grammar"
}

# ── job 2: Cypherdoc sub-grammar ──────────────────────────────────────────────

job_cypherdoc() {
    section "Cypherdoc grammar"
    check_cmd node
    check_cmd npx

    cd "$REPO_ROOT/tree-sitter-cypherdoc"

    printf 'Installing npm dependencies...\n'
    npm ci --omit=peer --omit=optional --silent

    printf 'Regenerating parser...\n'
    TREE_SITTER_ABI_VERSION=15 npx tree-sitter generate

    printf 'Running corpus tests...\n'
    npx tree-sitter test

    cd "$REPO_ROOT"
    pass "Cypherdoc grammar"
}

# ── job 3: cypher-data CLI tool ───────────────────────────────────────────────

job_tool() {
    section "cypher-data CLI tool"
    check_cmd cargo

    printf 'Building...\n'
    cargo build -p cypher-data

    printf 'Testing...\n'
    cargo test -p cypher-data

    pass "cypher-data CLI tool"
}

# ── dispatch ──────────────────────────────────────────────────────────────────

TARGET="${1:-all}"

FAILED=0

run_job() {
    # `set -e` would exit on the inner failure before we can set FAILED=1,
    # so temporarily disable it around each job invocation.
    set +e
    "$1"
    status=$?
    set -e
    if [ $status -ne 0 ]; then
        fail "$2"
        FAILED=1
    fi
}

case "$TARGET" in
    grammar)  job_grammar  ;;
    doc)      job_cypherdoc ;;
    tool)     job_tool     ;;
    all)
        run_job job_grammar  "Cypher grammar"
        run_job job_cypherdoc "Cypherdoc grammar"
        run_job job_tool     "cypher-data CLI tool"
        ;;
    *)
        printf 'Usage: %s [grammar|doc|tool|all]\n' "$0" >&2
        exit 1
        ;;
esac

if [ "$FAILED" -eq 0 ]; then
    printf '\n\033[1;32mAll checks passed.\033[0m\n'
else
    printf '\n\033[1;31mOne or more checks failed.\033[0m\n' >&2
    exit 1
fi
