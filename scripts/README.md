Scripts Overview

- test-all.sh / test-all.ps1
  - Runs workspace tests for Rust, optional Hardhat contract tests, and optional WebAssembly tests.
  - Supports starting local infra via docker compose (MySQL, Redis, MongoDB).
  - Common examples:
    - `bash scripts/test-all.sh --start-infra`
    - `bash scripts/test-all.sh --package p-project-dao --skip-contracts --skip-web`
    - `pwsh -File scripts/test-all.ps1 -StartInfra -NoContracts -NoWeb`

- deploy-images.sh / deploy-images.ps1
  - Builds Docker images for api, web, relayer, and airdrop components; optionally pushes to GHCR.
  - Mirrors the GitHub Actions workflow tags and names.
  - Examples:
    - `bash scripts/deploy-images.sh --component api --tag dev --push`
    - `pwsh -File scripts/deploy-images.ps1 -Component all -Tag sha-123 -Push`

- deploy-k8s.sh / deploy-k8s.ps1
  - Applies k8s manifests under `k8s/` and optionally overrides container images per deployment.
  - Examples:
    - `bash scripts/deploy-k8s.sh --namespace staging --api-image ghcr.io/acme/p-project-api:sha-123`
    - `pwsh -File scripts/deploy-k8s.ps1 -Namespace prod -WebImage ghcr.io/acme/p-project-web:latest`

Testing

- Linux: `bash scripts/tests/test_runner.sh`
- Windows: `pwsh -File scripts/tests/win-tests.ps1`

Notes

- Use `--dry-run` (bash) or `-DryRun` (PowerShell) to preview actions without executing.
- For pushing to GHCR, authenticate first: `echo $PAT | docker login ghcr.io -u USERNAME --password-stdin`.
- WebAssembly tests require a headless Chrome/Chromium; when unavailable, the wasm build still runs and tests are skipped.

