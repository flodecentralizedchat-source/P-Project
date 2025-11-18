#!/usr/bin/env bash
set -euo pipefail

# Build and optionally push Docker images for components.
# Mirrors logic in .github/workflows/docker-images.yml for local/CI usage.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

COMPONENT="all"
PUSH=0
TAG=""
OWNER=""
DRY_RUN=0

usage() {
  cat <<EOF
Usage: $(basename "$0") [options]

Options:
  --component <name>   One of: api, web, relayer, airdrop, all (default: all)
  --push               Push to GHCR (requires docker login)
  --tag <tag>          Image tag (default: sha-<git-sha>)
  --owner <owner>      GHCR owner/org (default: repo owner env)
  --dry-run            Print actions without executing
  -h, --help           Show this help

Examples:
  $(basename "$0") --component api --tag dev --push
  $(basename "$0") --component web --dry-run
EOF
}

log() { printf "[deploy-images] %s\n" "$*"; }
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
    --component) COMPONENT="$2"; shift 2 ;;
    --push) PUSH=1; shift ;;
    --tag) TAG="$2"; shift 2 ;;
    --owner) OWNER="$2"; shift 2 ;;
    --dry-run) DRY_RUN=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 2 ;;
  esac
done

if [[ -z "$TAG" ]]; then
  if [[ -n "${GITHUB_SHA:-}" ]]; then
    TAG="sha-${GITHUB_SHA}"
  else
    TAG="sha-$(git rev-parse --short HEAD 2>/dev/null || echo local)"
  fi
fi

if [[ -z "$OWNER" ]]; then
  if [[ -n "${GITHUB_REPOSITORY_OWNER:-}" ]]; then
    OWNER="${GITHUB_REPOSITORY_OWNER,,}"
  else
    OWNER="example"
  fi
fi

require_cmd docker

build_one() {
  local name="$1" dockerfile="$2" image="$3"
  local context="$REPO_ROOT"
  local base="docker buildx build --platform linux/amd64 --file '$dockerfile' '$context' --tag '$image:$TAG'"
  if [[ "$PUSH" -eq 1 ]]; then
    run "$base --push"
  else
    # --load supports single arch; fine for local dev
    run "$base --load"
  fi
}

case "$COMPONENT" in
  api)
    build_one api "$REPO_ROOT/p-project-api/Dockerfile" "ghcr.io/$OWNER/p-project-api" ;;
  web)
    build_one web "$REPO_ROOT/p-project-web/Dockerfile" "ghcr.io/$OWNER/p-project-web" ;;
  relayer)
    build_one relayer "$REPO_ROOT/p-project-bridge/Dockerfile" "ghcr.io/$OWNER/p-project-bridge-relayer" ;;
  airdrop)
    build_one airdrop "$REPO_ROOT/p-project-airdrop/Dockerfile" "ghcr.io/$OWNER/p-project-airdrop-cron" ;;
  all)
    build_one api "$REPO_ROOT/p-project-api/Dockerfile" "ghcr.io/$OWNER/p-project-api"
    build_one web "$REPO_ROOT/p-project-web/Dockerfile" "ghcr.io/$OWNER/p-project-web"
    build_one relayer "$REPO_ROOT/p-project-bridge/Dockerfile" "ghcr.io/$OWNER/p-project-bridge-relayer"
    build_one airdrop "$REPO_ROOT/p-project-airdrop/Dockerfile" "ghcr.io/$OWNER/p-project-airdrop-cron" ;;
  *)
    echo "Invalid component: $COMPONENT" >&2
    exit 2 ;;
esac

log "Done."

