# Performance Audit Report Template

Deliver as `PERF_AUDIT.md` at the repo root (or inline in chat for small audits). Keep the whole report skimmable in 5 minutes; depth lives in the per-finding details.

```markdown
# Performance Audit — <project> — <date>

## Executive summary
3–6 sentences: overall health, the single biggest bottleneck, expected gains
from the quick wins. No jargon a PM couldn't follow.

## Scorecard
| Area | Status | Biggest issue |
|---|---|---|
| Database access | 🔴/🟡/🟢 | one line |
| Caching | ... | |
| Request path / queues | | |
| Frontend bundle | | |
| Rendering & data fetching | | |
| Assets & delivery | | |

## Quick wins (do these first)
Findings with Small effort and High/Medium impact, as a checklist with links
to the full findings below.

## Findings

### P1 — <title> (Impact: High · Effort: S/M/L)
**Where:** `path/file.php:123` (+ route/page affected)
**Evidence:** the query / measurement / bundle stat proving it
**Why it's slow:** one short paragraph
**Fix:** concrete — the code change, index DDL, or config, ready to apply
**Expected gain:** honest estimate ("~40 queries → 2 per request")

### P2 — ...

## Later / not now
One line each: theoretical-scale concerns, L-effort refactors, things
already mitigated. Ranked out deliberately, not forgotten.

## Suggested guardrails
Regression protection worth adding: slow-query log threshold, bundle-size
budget in CI, Lighthouse CI budget, preventLazyLoading in dev.
```

## Ranking rules

- P1 = user-facing hot path or cost driver, fix well-understood. P2 = meaningful but narrower. P3 = worthwhile hygiene.
- Within a priority, order by impact/effort ratio.
- Max ~10 full findings; everything else goes to "Later" — a 40-finding report gets zero findings fixed.
- Never present an estimate as a measurement. Measured numbers get the measurement context (local, N rows, cold cache).
