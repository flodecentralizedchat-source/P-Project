#!/usr/bin/env bash
set -euo pipefail

# Cross-platform test runner for the monorepo
# - Runs Rust tests across workspace (optionally filtered by package)
# - Optionally runs Hardhat contract tests
# - Optionally runs WebAssembly tests/builds
# - Can optionally start local infra via Docker Compose (MySQL/Redis/MongoDB)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RUST=1
CONTRACTS=1
WEB=0
START_INFRA=0
PACKAGE=""
CONTRACTS_DIR="$REPO_ROOT/p-project-contracts/src/contracts"
DRY_RUN=0

print_usage() {
  cat <<EOF
Usage: $(basename "$0") [options]

Options:
  --rust/--skip-rust           Run or skip Rust tests (default: run)
  --contracts/--skip-contracts Run or skip Hardhat contract tests (default: run)
  --web/--skip-web             Run or skip WebAssembly tests (default: skip)
  --start-infra                Start MySQL/Redis/MongoDB via docker compose
  --package <name>             Run cargo tests for a specific package
  --contracts-dir <path>       Override contracts directory (default: $CONTRACTS_DIR)
  --dry-run                    Print actions without executing them
  -h, --help                   Show this help

Examples:
  $(basename "$0") --start-infra
  $(basename "$0") --package p-project-dao --skip-contracts --skip-web
  $(basename "$0") --web  # run wasm tests (requires headless browser)
EOF
}

log() { printf "[test-all] %s\n" "$*"; }
run() { if [[ "$DRY_RUN" -eq 1 ]]; then echo "+ $*"; else eval "$*"; fi; }

require_cmd() {
  local name="$1"
  if [[ "$DRY_RUN" -eq 1 ]]; then return 0; fi
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "Error: required command not found: $name" >&2
    exit 127
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --rust) RUST=1; shift ;;
    --skip-rust) RUST=0; shift ;;
    --contracts) CONTRACTS=1; shift ;;
    --skip-contracts) CONTRACTS=0; shift ;;
    --web) WEB=1; shift ;;
    --skip-web) WEB=0; shift ;;
    --start-infra) START_INFRA=1; shift ;;
    --package) PACKAGE="$2"; shift 2 ;;
    --contracts-dir) CONTRACTS_DIR="$2"; shift 2 ;;
    --dry-run) DRY_RUN=1; shift ;;
    -h|--help) print_usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; print_usage; exit 2 ;;
  esac
done

if [[ "$START_INFRA" -eq 1 ]]; then
  log "Starting local infra (docker compose: mysql, redis, mongodb)"
  require_cmd docker
  run "docker compose -f '$REPO_ROOT/docker-compose.yml' up -d mysql redis mongodb"
fi

if [[ "$RUST" -eq 1 ]]; then
  log "Running Rust tests"
  require_cmd cargo
  local_env="DATABASE_URL=\"mysql://root:rootpassword@localhost:3306/p_project\" \
REDIS_URL=\"redis://localhost:6379\" \
MONGODB_URL=\"mongodb://localhost:27017\" \
MONGO_URI=\"mongodb://localhost:27017\" \
MONGO_DB=\"p_project_dao_test_ci\""
  if [[ -n "$PACKAGE" ]]; then
    run "$local_env cargo test -p '$PACKAGE' --all-features --verbose"
  else
    run "$local_env cargo test --all-features --workspace --verbose"
    # Known ignored test in CI; keep parity
    run "$local_env cargo test -p p-project-dao -- --ignored create_and_fetch_active_proposal_via_mongo"
  fi
fi

if [[ "$CONTRACTS" -eq 1 ]]; then
  log "Running Hardhat contract tests at: $CONTRACTS_DIR"
  if [[ ! -d "$CONTRACTS_DIR" ]]; then
    echo "Contracts directory not found: $CONTRACTS_DIR" >&2
    exit 1
  fi
  require_cmd npx
  run "cd '$CONTRACTS_DIR' && npx hardhat compile"
  run "cd '$CONTRACTS_DIR' && npx hardhat test"
fi

if [[ "$WEB" -eq 1 ]]; then
  log "Running WebAssembly build and tests"
  require_cmd wasm-pack
  # Build web pkg
  run "wasm-pack build '$REPO_ROOT/p-project-web' --target web"
  # wasm tests require a browser runner; attempt headless chrome if available
  if command -v google-chrome >/dev/null 2>&1 || command -v chromium >/dev/null 2>&1 || command -v chrome >/dev/null 2>&1; then
    run "wasm-pack test '$REPO_ROOT/p-project-web' --headless --chrome"
  else
    log "No headless Chrome found; skipping wasm tests (build done)."
  fi
fi

log "Done."

