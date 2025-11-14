# Security Policy

We take security seriously and appreciate responsible disclosures.

## Supported Versions

We aim to fix security issues on the `main` branch and the latest tagged release. Older releases may not receive fixes.

## Reporting a Vulnerability

Please do not open public issues for security vulnerabilities.

- If your repository has GitHub’s “Private vulnerability reporting” enabled, use the “Report a vulnerability” button under the “Security” tab.
- Otherwise, open a GitHub Security Advisory draft (preferred) or contact the maintainers privately.

Provide as much detail as possible:

- Affected component(s) and version/commit
- Steps to reproduce and impact assessment
- Any proofs-of-concept (attach privately)

We will acknowledge receipt within 3 business days and provide a remediation timeline after triage.

## Scope

- Application code in this repository
- GitHub Actions workflows and deployment manifests in this repository

Third‑party dependencies and upstream images are out of direct scope, but we track and fix them via automated scanning (CodeQL, Trivy, Grype, cargo‑audit/deny, Dependency Review) and weekly Dependabot updates.

## Coordinated Disclosure

We prefer coordinated disclosure. We will credit reporters in release notes unless you request otherwise.

