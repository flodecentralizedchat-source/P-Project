#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

pass() { echo "[PASS] $*"; }
fail() { echo "[FAIL] $*"; exit 1; }

# test-all.sh help
out=$(bash "$ROOT/scripts/test-all.sh" --help)
echo "$out" | grep -q "Usage:" || fail "test-all.sh --help did not print usage"
pass "test-all.sh --help"

# test-all.sh dry-run
bash "$ROOT/scripts/test-all.sh" --dry-run --start-infra --skip-contracts --skip-web || fail "test-all.sh --dry-run failed"
pass "test-all.sh --dry-run"

# deploy-images.sh help
out=$(bash "$ROOT/scripts/deploy-images.sh" --help)
echo "$out" | grep -q "Usage:" || fail "deploy-images.sh --help did not print usage"
pass "deploy-images.sh --help"

# deploy-images.sh dry-run
bash "$ROOT/scripts/deploy-images.sh" --dry-run --component api --tag ci-test || fail "deploy-images.sh --dry-run failed"
pass "deploy-images.sh --dry-run"

# deploy-k8s.sh help
out=$(bash "$ROOT/scripts/deploy-k8s.sh" --help)
echo "$out" | grep -q "Usage:" || fail "deploy-k8s.sh --help did not print usage"
pass "deploy-k8s.sh --help"

# deploy-k8s.sh dry-run
bash "$ROOT/scripts/deploy-k8s.sh" --dry-run --namespace ci --api-image ghcr.io/example/p-project-api:test || fail "deploy-k8s.sh --dry-run failed"
pass "deploy-k8s.sh --dry-run"

echo "All script tests passed."

