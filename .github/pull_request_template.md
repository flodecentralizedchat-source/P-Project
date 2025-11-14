Title: <short summary>

Summary
- What changed and why?
- Any user-visible behavior or API changes?

Security & Privacy Checklist
- [ ] No secrets or credentials added to code, configs, or logs
- [ ] Inputs validated and bounded (lengths, enums, numeric ranges)
- [ ] Authn/authz implications considered (no public endpoints added without controls)
- [ ] Errors do not leak sensitive data
- [ ] Cryptography and randomness use safe primitives (no MD5/SHA1 for security)
- [ ] Monetary amounts use Decimal, not floats (unless purely for display)

Build & CI
- [ ] cargo fmt/clippy clean locally
- [ ] cargo test passes locally (if tests exist)
- [ ] CI scanners (audit/deny/gitleaks) expected to pass

Testing
- How was this tested? Include manual steps or test links.

Screenshots (optional)

