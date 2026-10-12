---
name: perf-audit
description: Run a systematic performance audit on a codebase and produce a prioritized, evidence-based report with concrete fixes. Use this skill whenever the user asks about performance — "audit performance", "why is this slow", "find bottlenecks", "optimize this app", "performance review", "reduce load time", "the API is slow", "check for N+1 queries", "bundle is too big" — or asks to review a codebase for speed, scalability, memory, or resource usage, even if they don't say the word "audit".
---

# Performance Audit

Find real bottlenecks, prove them with evidence, and rank fixes by impact-over-effort. The failure mode to avoid: a laundry list of micro-optimizations while the one query doing 4,000 round-trips goes unmentioned.

## Ground rules

- **Evidence over vibes.** Every finding cites a file:line, a query, a bundle stat, or a measurement. "This looks slow" is not a finding. If you can run the code, measure; if you can't, reason from the data flow and label it as static-analysis confidence.
- **Hot paths first.** A 10× speedup on a route nobody hits is worth less than 10% on the login page. Establish which paths matter (traffic, user-facing latency, cost) before diving deep.
- **Don't confuse perf with style.** Readability nits, naming, and architecture opinions belong in a code review, not here — unless the structure itself causes the slowness.
- **Estimate, honestly.** Each finding gets an impact estimate (order of magnitude is fine: "~N queries per request → 1", "~300 KB gzip off the main bundle") and an effort estimate (S/M/L). Say when you're unsure.

## Workflow

### 1. Scope (5 min)
Ask or infer: what feels slow (specific pages/endpoints or general)? What's the environment (traffic scale, hosting, perf budget)? If the user named a symptom, the audit is symptom-first; otherwise do a full sweep.

### 2. Map the stack
Read manifests, framework config, and infra files (Dockerfile, CI, `.env.example`) to identify: framework + version, ORM, cache layers present (or absent), queue usage, frontend build setup. Then read the matching reference file(s):

- PHP / Laravel backend → `references/php-laravel.md`
- Next.js / React / general JS frontend → `references/js-frontend.md`
- Any app with a relational DB → `references/database.md` (read this in almost every audit)

### 3. Sweep the hot paths
For each area, work through the checklist in its reference file against the actual code. Prioritize: routes/controllers/pages with the most traffic or user-facing latency, then jobs/crons that touch lots of data. Trace a request end-to-end at least once (route → middleware → controller → queries → serialization → response) — cross-layer waste (e.g., loading full models to render three fields) only shows up in end-to-end traces.

### 4. Measure what you can
If the environment allows: run the test suite with query logging, `EXPLAIN` suspicious queries, build the frontend and read the bundle analyzer output, time a representative request. Prefer the project's own tooling (Telescope/Debugbar, `next build` stats, Lighthouse CI) over installing new ones. Never run load tests against production.

### 5. Report
Write the report using the structure in `references/report-template.md`: executive summary, findings ranked by impact/effort, each with evidence + concrete fix (code-level, not "consider caching"). Quick wins (S effort, high impact) get their own section at the top.

### 6. Offer next steps
Offer to implement the top quick wins, or to set up guardrails so regressions get caught: slow-query log threshold, bundle-size budget in CI, Lighthouse CI. Don't implement unasked.

## What NOT to flag

- Premature optimization in cold paths (a `foreach` vs `array_map` in a CLI setup script)
- Theoretical scaling problems far beyond the project's stated scale — note them in one line under "Later", don't rank them
- Anything already behind a cache with a sane TTL, unless the invalidation is broken
