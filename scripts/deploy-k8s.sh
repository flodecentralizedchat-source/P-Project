#!/usr/bin/env bash
set -euo pipefail

# Apply Kubernetes manifests and optionally override images per deployment.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

NAMESPACE="default"
API_IMAGE=""
WEB_IMAGE=""
RELAYER_IMAGE=""
AIRDR0P_IMAGE=""  # typo-protected variable to avoid shellcheck confusion
AIR_DROP_IMAGE=""
DRY_RUN=0

usage() {
  cat <<EOF
Usage: $(basename "$0") [options]

Options:
  --namespace <name>       Kubernetes namespace (default: default)
  --api-image <ref>        Override API image
  --web-image <ref>        Override Web image
  --relayer-image <ref>    Override Bridge Relayer image
  --airdrop-image <ref>    Override Airdrop Cron image
  --dry-run                Print actions without executing
  -h, --help               Show this help

Examples:
  $(basename "$0") --namespace staging \
    --api-image ghcr.io/acme/p-project-api:sha-123 \
    --web-image ghcr.io/acme/p-project-web:sha-123
EOF
}

log() { printf "[deploy-k8s] %s\n" "$*"; }
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
    --namespace) NAMESPACE="$2"; shift 2 ;;
    --api-image) API_IMAGE="$2"; shift 2 ;;
    --web-image) WEB_IMAGE="$2"; shift 2 ;;
    --relayer-image) RELAYER_IMAGE="$2"; shift 2 ;;
    --airdrop-image) AIR_DROP_IMAGE="$2"; shift 2 ;;
    --dry-run) DRY_RUN=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 2 ;;
  esac
done

require_cmd kubectl

# Apply manifests
for f in "$REPO_ROOT/k8s/api.yaml" "$REPO_ROOT/k8s/web.yaml" "$REPO_ROOT/k8s/bridge-relayer.yaml" "$REPO_ROOT/k8s/airdrop-cronjob.yaml"; do
  if [[ -f "$f" ]]; then
    run "kubectl apply -n '$NAMESPACE' -f '$f'"
  else
    log "Skipping missing manifest: $f"
  fi
done

# Optional image overrides
if [[ -n "$API_IMAGE" ]]; then
  run "kubectl -n '$NAMESPACE' set image deploy/p-project-api api='$API_IMAGE' --record=true"
fi
if [[ -n "$WEB_IMAGE" ]]; then
  run "kubectl -n '$NAMESPACE' set image deploy/p-project-web web='$WEB_IMAGE' --record=true"
fi
if [[ -n "$RELAYER_IMAGE" ]]; then
  run "kubectl -n '$NAMESPACE' set image deploy/p-project-bridge-relayer relayer='$RELAYER_IMAGE' --record=true"
fi
if [[ -n "$AIR_DROP_IMAGE" ]]; then
  # CronJobs need a different update path; patch the image in the template
  run "kubectl -n '$NAMESPACE' set image cronjob/p-project-airdrop-cron airdrop-cron='$AIR_DROP_IMAGE' --record=true"
fi

log "Done."
