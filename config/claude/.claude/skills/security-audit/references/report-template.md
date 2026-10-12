# Security Audit Report Template

Deliver as `SECURITY_AUDIT.md` (warn the user: this file is a roadmap for attackers — keep it out of the public repo until findings are fixed) or inline for small audits.

```markdown
# Security Audit — <project> — <date>

## Executive summary
3–6 sentences: overall posture, count by severity, the one thing to fix
today. Include scope + method (static review, tooling run, versions checked)
and what was NOT covered (infra, DoS, social engineering) — honesty about
scope is part of the deliverable.

## Severity overview
| Severity | Count | Titles |
|---|---|---|
| Critical | n | ... |
| High | n | ... |
| Medium | n | ... |
| Low / Info | n | ... |

## Findings

### [C-1] <title> — Critical
**Where:** `app/Http/Controllers/X.php:42` · route `POST /api/things/{id}`
**Class:** e.g. Broken access control (IDOR)
**Evidence:** the tainted path, source → sink, or the request that
demonstrates it (confirmation-level only, no weaponized payloads)
**Impact:** what an attacker gets, concretely, in this app
**Fix:** the code/config change, ready to apply
**Verify:** the request/test that should now fail (403, escaped output…)

### [H-1] ...

## Hardening (defense in depth — not vulnerabilities)
Headers, CSP rollout, rate limits, MFA, audit logging, secret scanning in
CI, Dependabot. Checklist form, each with a one-line rationale.

## Out of scope / noted only
Dev-only tooling issues, unreachable theoreticals, EOL warnings already
scheduled. One line each.

## Suggested guardrails
CI: dependency audit, gitleaks, regression tests for each fixed finding.
```

## Rules

- Order findings by severity, then by fix effort within a severity.
- IDs (`C-1`, `H-2`) so fixes/PRs can reference them.
- Rotated-credential findings: state explicitly that rotation is the fix and removal from history is secondary.
- If NOTHING serious was found, say so plainly and resist inflating Medium findings to justify the audit — a short honest report builds more trust.
- Remind the user an AI static review is not a substitute for a professional pentest when the stakes warrant one (payments, health data, compliance).
