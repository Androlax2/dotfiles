# Secrets, Dependencies & Delivery Config

Stack-independent sweep. Run in every audit — committed secrets are the most common Critical finding in real codebases.

## 1. Secrets

- **Working tree**: grep for high-entropy strings and known prefixes (`sk_live`, `AKIA`, `ghp_`, `xoxb`, `-----BEGIN`, `eyJ` JWTs) across code, config, docker-compose, CI files, and seeds/fixtures.
- **Git history**: a secret removed in a later commit is still compromised. `git log -p` spot checks on env/config paths, or run `gitleaks detect` if available. Finding = rotate the credential; history rewriting is secondary and optional.
- **Client bundles**: build output and public env vars (see stack files) — anything shipped to the browser is public.
- Docker: secrets baked into image layers via `ARG`/`COPY .env`; CI logs echoing env.
- Fix pattern: rotate first, then move to env/secret manager, then add gitleaks or GitHub secret scanning + push protection as a guardrail.

## 2. Dependencies

- Run the ecosystem audit (`composer audit`, `npm audit`/`pnpm audit`). Triage, don't dump: runtime dependencies with Critical/High and a reachable code path → Findings; dev-only or unreachable → one Hardening line.
- Framework/runtime versions vs EOL (PHP, Node, Laravel major, Next major) — EOL means no patches: severity scales with exposure.
- Lockfiles committed and installs frozen in CI (`--frozen-lockfile` / `composer install` not `update`).
- Update path: suggest Dependabot/Renovate as a guardrail if absent.

## 3. HTTP delivery (Hardening section material)

Check at the framework middleware or reverse-proxy level:

- `Strict-Transport-Security` (with sane max-age; preload only if they mean it)
- `X-Content-Type-Options: nosniff`
- Clickjacking: `frame-ancestors` (CSP) or `X-Frame-Options` on authenticated UI
- `Referrer-Policy: strict-origin-when-cross-origin` (or stricter)
- CSP: recommend report-only rollout for existing apps; enforce for new ones. Escalate from Hardening → part of a Finding's fix when XSS was found.
- Cookies re-checked at the edge: `Secure`, `HttpOnly`, `SameSite` actually present in responses (config can lie when a proxy rewrites).
- CORS: explicit origins, no `*` with credentials, methods/headers minimal, preflight not bypassed by "reflect the Origin header" code.

## 4. Operational surface

- Debug/monitoring endpoints exposed: framework debug pages, `/telescope`, `/horizon`, `/_profiler`, GraphQL introspection + playground in prod, metrics endpoints without auth.
- Directory listings, `.git/` served, source maps in production (info leak — Low unless secrets inside), backup files (`.sql`, `.zip`) in webroot.
- Error responses: consistent 404 vs 403 policy (existence leaks), stack traces off.
- Logging: auth events logged (login success/failure, password change) WITHOUT sensitive payloads; log injection via CRLF in user input if logs feed other systems.
- Backups & uploads buckets: public-read cloud storage misconfig if infra config is in repo (terraform, S3 policies).
