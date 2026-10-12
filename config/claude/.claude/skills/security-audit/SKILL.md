---
name: security-audit
description: Run a defensive security audit on a codebase and produce a severity-ranked report with concrete remediations. Use this skill whenever the user asks to audit, review, or harden a codebase for security — "security audit", "is this safe", "check for vulnerabilities", "pentest my code", "review auth", "am I vulnerable to SQL injection/XSS", "harden this before launch", "OWASP check" — or before open-sourcing or deploying an app publicly, even if they don't say the word "security".
---

# Security Audit

Find vulnerabilities in the user's own codebase, prove them with evidence, and hand over fixes. This is defensive work on code the user owns or maintains: the output is remediation, never working exploits against third parties.

## Ground rules

- **Evidence per finding.** Every finding cites file:line plus the tainted data path (source → sink). "Inputs should be validated" is not a finding; "`$request->q` flows unescaped into `DB::raw()` at `SearchController.php:42`" is.
- **Severity honestly.** Rate by exploitability × impact in THIS app's context, not by category reputation. A reflected XSS on an admin-only debug page is not Critical. Use: **Critical** (remote compromise, auth bypass, data breach reachable now) / **High** / **Medium** / **Low** / **Info**.
- **A finding needs a fix.** Concrete, code-level remediation ready to apply — the parameterized query, the middleware, the config line. Include the verification step ("after the fix, this request should return 403").
- **Assume the framework, verify the escape hatches.** Modern frameworks are secure by default; nearly all real vulnerabilities hide where defaults were bypassed (`raw()`, `dangerouslySetInnerHTML`, `{!! !!}`, `eval`, manual SQL, disabled middleware). Grep for the escape hatches first.
- **Don't produce weaponization.** Proof-of-concept payloads to confirm a finding in the user's own app are fine (`' OR 1=1--`, an `alert(1)` XSS probe). Building out exploitation beyond confirmation — exfil scripts, payload obfuscation, attacks on infrastructure the user doesn't own — is out of scope; decline that portion and continue the audit.

## Workflow

### 1. Scope
What's the app's exposure (public internet, internal, pre-launch)? What data does it hold (payments, PII, health)? Any specific worry, or full sweep? Exposure + data sensitivity calibrate every severity rating.

### 2. Map the attack surface
Enumerate before judging: routes/endpoints (public vs authenticated vs admin), input channels (forms, JSON APIs, file uploads, webhooks, CLI, cron), outbound calls (SSRF surface), auth mechanisms, and where secrets/config live. List unauthenticated endpoints explicitly — they're the priority queue.

### 3. Sweep
Read `references/core-checklist.md` and work through it against the mapped surface. Then read the stack file(s) that apply:

- PHP / Laravel → `references/php-laravel.md`
- Next.js / React / Node → `references/js-nextjs.md`

Always finish with `references/secrets-and-dependencies.md` (repo history, env handling, dependency CVEs, headers/CORS) — it's stack-independent and yields the most common Critical (committed secrets).

### 4. Verify what you can
Run the project's own tooling where available: `composer audit` / `npm audit` (or `pnpm audit`), existing static analyzers (Psalm/PHPStan security rules, ESLint security plugins), and grep-based taint spot checks. Confirm suspected issues by reading the full data path, not by attacking a live system. Never test against production or any system the user doesn't own.

### 5. Report
Use `references/report-template.md`: executive summary, severity-ranked findings with source→sink evidence and fixes, and a hardening section for defense-in-depth items that aren't vulnerabilities (headers, rate limits, audit logging).

### 6. Offer next steps
Offer to implement Critical/High fixes, add regression tests for fixed issues, and wire guardrails: dependency audit in CI, secret scanning (gitleaks), security headers middleware. Don't implement unasked.

## What NOT to flag

- Theoretical issues with no reachable path in this app (rank as Info at most, one line)
- Defense-in-depth absences dressed up as vulnerabilities — headers and rate limits go in Hardening, not Findings, unless their absence enables a concrete attack found elsewhere
- Vulnerabilities in dev-only tooling never deployed (note in one line, don't rank)
