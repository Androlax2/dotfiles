# Core Checklist (stack-independent)

Work through each class against the mapped attack surface. For each, the method is the same: find the sinks, then trace whether untrusted input reaches them.

## 1. Injection

- **SQL**: raw query APIs, string-built SQL, interpolated identifiers (table/column names can't be parameterized — must be allowlisted). ORM ≠ immunity: check `raw`/`whereRaw`-style escape hatches and `ORDER BY` built from request input.
- **Command**: any shell exec function receiving request data, including "safe-looking" filenames. Fix: argument arrays / escaping functions, or remove the shell entirely.
- **Path traversal**: user input in file paths (`../`), including archive extraction (zip-slip) and template names. Fix: resolve + verify prefix against an allowlisted base dir.
- **Template/code injection**: user input compiled into templates, `eval`, dynamic includes, unserializing untrusted data (PHP `unserialize`, Node `vm`, pickle-alikes).
- **NoSQL/LDAP/XPath** if those stores exist: operator injection via unvalidated object input (`{"$gt": ""}`).

## 2. XSS

- Server templates: unescaped-output syntax with user data; user data in `<script>` blocks, event handlers, `href`/`src` (`javascript:` URLs), or CSS.
- Client: `innerHTML`-family sinks, sanitizer absent or homegrown (→ DOMPurify), user-controlled URLs rendered into links.
- Stored vs reflected: anything persisted and re-rendered (comments, names, profile fields) is the priority.
- JSON embedded in HTML pages without proper encoding (`</script>` breakout).

## 3. Broken access control (the #1 real-world class)

- **IDOR**: every route with an ID parameter — is ownership checked, or just authentication? `find($id)` vs `user->things()->findOrFail($id)` patterns. Check UPDATE/DELETE, not just reads; check the API twin of every web route.
- **Missing function-level checks**: admin routes protected only by hidden UI; authorization done in the controller for some verbs but not others; mass assignment flipping privilege fields (`is_admin`, `role`) — check unguarded model binding / spread of request body into updates.
- **Multi-tenancy**: any query missing the tenant scope; global scopes bypassed by `withoutGlobalScopes`.
- Direct object references in URLs to files (uploads served by predictable path with no auth check).

## 4. Authentication & sessions

- Password storage: modern adaptive hash (bcrypt/argon2) via the framework — flag any md5/sha*, homegrown crypto, or plaintext anywhere (including logs).
- Login: rate limiting/lockout present? User enumeration via different error messages or timing on login/reset/registration?
- Password reset: tokens single-use, expiring, hashed at rest; reset flow doesn't leak account existence.
- Session: regenerated on login (fixation), invalidated on logout/password change; cookies `HttpOnly`, `Secure`, `SameSite=Lax`+.
- Tokens/JWT if present: algorithm pinned (no `none`/`HS256⇄RS256` confusion), expiry enforced, revocation story exists, secrets strong and not in repo.
- Remember-me tokens rotated; MFA if the data sensitivity warrants it (Hardening item otherwise).

## 5. CSRF & state-changing requests

- Framework CSRF middleware on all state-changing web routes; every exclusion (`VerifyCsrfToken::$except`-style lists) justified. GET routes that mutate state are findings by themselves.
- Cookie-authenticated APIs (same-site SPAs) still need CSRF protection or strict `SameSite` — bearer-token APIs generally don't; don't flag ritually.

## 6. SSRF & outbound requests

- Any fetch/HTTP call whose URL derives from user input (webhooks, "import from URL", PDF/screenshot generators): scheme + host allowlisting, deny private/link-local ranges (169.254.169.254 cloud metadata), disable redirects-to-anywhere.
- XML parsing with external entities enabled (XXE).

## 7. File uploads

- Validation by content (MIME sniff + extension allowlist), not client Content-Type alone; size limits; images re-encoded if feasible.
- Storage outside the webroot or behind an auth-checked download route; served with `Content-Disposition`/correct type so uploads can't execute or become stored XSS (SVG!).
- Filenames regenerated server-side (traversal + collision).

## 8. Crypto & sensitive data

- TLS assumed at the edge; flag plain-HTTP calls to internal/external services carrying credentials.
- Secrets/PII in logs, error messages, or stack traces exposed to users (debug mode in prod).
- Encryption misuse: ECB mode, static IVs, homegrown constructions → framework crypto primitives.

## 9. Business logic (read the flows, not just the sinks)

- Race conditions on balance/inventory/coupon operations (check for atomic updates or locks).
- Negative/absurd quantities and amounts server-validated, prices taken from DB not request.
- Workflow skips: can a later step be called directly without the earlier ones?
