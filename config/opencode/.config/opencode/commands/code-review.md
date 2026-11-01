---
description: Review the current diff, or a PR/branch/path target, for correctness bugs plus reuse, simplification and efficiency cleanups at an effort level (low|medium|high|xhigh|max)
subagent: true
---

Arguments: $ARGUMENTS

This is a port of Claude Code 2.1.316's `/code-review`. Follow it exactly.

## Read the arguments

Arguments have the shape `[low|medium|high|xhigh|max] [--fix] [--comment] [<pr#>|<branch>|<path>] [--max-findings <n>|all]`.

- **Level**: the first word, when it is one of `low`, `medium`, `high`, `xhigh`, `max`.
  With no level, use `medium` and open your report with one short line saying so, and
  that typing a level (for example `/code-review high`) changes it. A first word that
  looks like a level but is not one (a word starting with `low`, `med`, `hig`, `xhi`
  or `max`, such as `med`): open your report with "Ignoring unrecognized effort
  "<word>"; valid: low, medium, high, xhigh, max. Using medium." and use `medium`.
  The word stays part of the target.
- **`--max-findings <n>`**: a whole number above zero replaces the level's cap below;
  `all` removes it; `default` keeps the usual cap. Anything else: say in one line that `--max-findings` was ignored
  (type a whole number above zero or `all` after it) and use the usual cap.
- **`--fix`** and **`--comment`**: see the sections at the end.
- **Target**: what remains. Review it instead of the current diff.

Then follow the recipe for the level, with its cap:

| Level | Recipe | Cap |
| --- | --- | --- |
| low | Low | 4 |
| medium, high | Angles | 10 |
| xhigh | Angles, extra-high | 15 |
| max | Angles, maximum | 15 |

## Recipe: Low

`low effort → 1 diff pass → no verify → ≤<cap> findings`

### Turn 1 — read

One tool call: read the unified diff (`git diff @{upstream}...HEAD; git diff HEAD`
to cover both committed and uncommitted changes, or `git diff main...HEAD` /
the target passed as an argument). Skip test/fixture
hunks (`test/`, `spec/`, `__tests__/`, `*_test.*`, `*.test.*`,
`fixtures/`, `testdata/`) — test-file changes are not reviewed at this level.
No subagents, no full-file reads.

### Turn 2 — findings

Flag runtime-correctness bugs visible from the hunk alone: inverted/wrong
condition, off-by-one, null/undefined deref where adjacent lines show the value
can be absent, removed guard, falsy-zero check, missing `await`,
wrong-variable copy-paste, error swallowed in a catch that should propagate.
Also flag — still from the hunk alone — new code that duplicates an existing
helper visible in the diff context, and dead code the diff leaves behind.

Do **not** flag style, naming, perf, missing tests, or anything outside the
hunk.

Output at most **<cap> findings** (with `--max-findings all`: every finding),
most-severe first, one line each:
`path/to/file.ext:123 — what's wrong and the concrete failure`. If nothing
qualifies, output exactly `(none)`.

## Recipe: Angles

`<level> effort → 8 inline angles → dedup (no verify) → ≤<cap> findings`

You are reviewing for **recall** at high effort: catch every real bug a careful
reviewer would catch in one sitting. At this level, catching real bugs matters
more than avoiding false positives. Err on the side of surfacing.

### Phase 0 — Gather the diff

Run `git diff @{upstream}...HEAD` (or `git diff main...HEAD` / `git diff HEAD~1`
if there's no upstream) to get the unified diff under review. If there are
uncommitted changes, or the range diff is empty, also run `git diff HEAD` and
include the working-tree changes in scope — the review often runs before the
commit. If a PR number, branch name, or file path was passed as an argument,
review that target instead. Treat this diff as the review scope.

### Phase 1 — Find candidates (3 correctness angles + 3 cleanup angles + 1 altitude angle + 1 conventions angle, up to 6 each)

Run **8 independent finder angles** in sequence yourself, in THIS context — do NOT spawn subagents for them. Each
surfaces **up to 6 candidate findings** with `file`, `line`, a one-line
`summary`, and a concrete `failure_scenario`.

Use angles A, B, C, then Reuse, Simplification, Efficiency, Altitude and
Conventions, from **The angles** below.

Pass every candidate with a nameable failure scenario through — finders that
silently drop half-believed candidates are the dominant cause of misses.

### Phase 2 — Dedup only (no verify)

Pool all candidates. Dedup near-duplicates only (same defect, same location, same reason → keep one). Do NOT run verifiers; do NOT re-judge. Sort by severity.

Then write the **Output** below.

## Recipe: Angles, extra-high

`<level> effort → 10 inline angles → dedup (no verify) → sweep → ≤<cap> findings`

You are reviewing for **recall** at extra-high effort: catch every real bug. At
this level, catching real bugs matters more than avoiding false positives — a
missed bug ships. Err on the side of surfacing.

### Phase 0 — Gather the diff

As in **Recipe: Angles**.

### Phase 1 — Find candidates (5 correctness angles + 3 cleanup angles + 1 altitude angle + 1 conventions angle, up to 8 each)

Run **10 independent finder angles** in sequence yourself, in THIS context — do NOT spawn subagents for them. Each
surfaces **up to 8 candidate findings**. Do NOT let one angle's conclusions
suppress another's — if two angles flag the same line for different reasons,
record both.

Use angles A, B, C, D, E, then Reuse, Simplification, Efficiency, Altitude and
Conventions, from **The angles** below.

### Phase 2 — Dedup only (no verify)

Pool all candidates. Dedup near-duplicates only (same defect, same location, same reason → keep one). Do NOT run verifiers; do NOT re-judge. Sort by severity. Do NOT drop on uncertainty.

### Phase 3 — Sweep for gaps

Take one more pass (same context — no subagent) as a fresh reviewer who has the deduplicated list. Re-read
the diff and enclosing functions looking ONLY for defects not already listed.
Do not re-derive or re-confirm anything already there — the job is gaps. Focus
on what the first pass tends to miss: moved/extracted code that dropped a guard
or anchor; second-tier footguns (dataclass default evaluated once, `hash()`
non-determinism, lock-scope shrink, predicate methods with side effects);
setup/teardown asymmetry in tests; config defaults flipped.

Surface **up to 8 additional candidates**, each naming a defect not already on
the list. If nothing new, return nothing from this phase — do not pad.

Then write the **Output** below.

## Recipe: Angles, maximum

`max effort → subagent tool unavailable → single-pass inline → ≤<cap> findings`

You are reviewing for **recall** at maximum effort: catch every real bug. At
this level, catching real bugs matters more than avoiding false positives — a
missed bug ships. Err on the side of surfacing.

The usual multi-agent fan-out and subagent verify pass can't run. Work through
every angle below yourself, in this same context, in one pass — do not skip angles
for lack of fan-out. Re-check each candidate against the diff before keeping
it; drop anything you can't back up with a concrete failure scenario.

### Phase 0 — Gather the diff

As in **Recipe: Angles**.

### Phase 1 — Find candidates (10 angles, single pass)

Work through **10 angles** yourself, in sequence, in this same
context — do not spawn subagents. Each surfaces candidate findings with
`file`, `line`, a one-line `summary`, and a concrete `failure_scenario`.

Use angles A, B, C, D, E, then Reuse, Simplification, Efficiency, Altitude and
Conventions, from **The angles** below.

### Phase 2 — Dedup and self-check (no subagent verify)

Dedup near-duplicates (same defect, same location, same reason → keep one).
Re-check each remaining candidate yourself against the diff before keeping it.

### Phase 3 — Sweep for gaps

Take one more pass yourself (same context, no subagent) as a fresh reviewer
who has the deduplicated list. Re-read the diff and enclosing functions
looking ONLY for defects not already listed: moved/extracted code that dropped a guard
or anchor; second-tier footguns (dataclass default evaluated once, `hash()`
non-determinism, lock-scope shrink, predicate methods with side effects);
setup/teardown asymmetry in tests; config defaults flipped.

Then write the **Output** below, and state clearly in your summary that this was
a single-pass review done without subagents, not the full multi-agent fan-out,
so whoever reads it isn't misled about what actually ran.

## The angles

### Angle A — line-by-line diff scan

Read every hunk in the diff, line by line. Then Read the enclosing function for
each hunk — bugs in unchanged lines of a touched function are in scope (the PR
re-exposes or fails to fix them). For every line ask: what input, state, timing,
or platform makes this line wrong? Look for inverted/wrong conditions,
off-by-one, null/undefined deref, missing `await`, falsy-zero checks,
wrong-variable copy-paste, error swallowed in catch, unescaped regex metachars.

### Angle B — removed-behavior auditor

For every line the diff DELETES or replaces, name the invariant or behavior it
enforced, then search the new code for where that invariant is re-established.
If you can't find it, that's a candidate: a removed guard, a dropped error
path, a narrowed validation, a deleted test that was covering a real case.

### Angle C — cross-file tracer

For each function the diff changes, find its callers (Grep for the symbol) and
check whether the change breaks any call site: a new precondition, a changed
return shape, a new exception, a timing/ordering dependency. Also check callees:
does a parallel change in the same PR make a call unsafe?

### Angle D — language-pitfall specialist

Scan for the classic pitfalls of the diff's language/framework — for example:
JS falsy-zero, `==` coercion, closure-captured loop var; Python mutable default
args, late-binding closures; Go nil-map write, range-var capture; SQL injection;
timezone/DST drift; float equality. Flag any instance the diff introduces.

### Angle E — wrapper/proxy correctness

When the PR adds or modifies a type that wraps another (cache, proxy, decorator,
adapter): check that every method routes to the wrapped instance and not back
through a registry/session/global — e.g. a caching provider holding a
`delegate` field that resolves IDs via `session.get(...)` instead of
`delegate.get(...)` will re-enter the cache or recurse. Also check that the
wrapper forwards all the methods the callers actually use.

### Reuse

The angles above hunt for bugs; this one and the next two hunt for cleanup in
the changed code. Flag new code that re-implements something the codebase
already has — Grep shared/utility modules and files adjacent to the change,
and name the existing helper to call instead.

### Simplification

Flag unnecessary complexity the diff adds: redundant or derivable state,
copy-paste with slight variation, deep nesting, dead code left behind. Name
the simpler form that does the same job.

### Efficiency

Flag wasted work the diff introduces: redundant computation or repeated I/O,
independent operations run sequentially, blocking work added to startup or
hot paths. Also flag long-lived objects built from closures or captured
environments — they keep the entire enclosing scope alive for the object's
lifetime (a memory leak when that scope holds large values); prefer a
class/struct that copies only the fields it needs. Name the cheaper
alternative.

### Altitude

Check that each change fixes the root cause at the right depth rather than
patching a symptom with a fragile bandaid. Special cases layered on shared
infrastructure are a sign the fix isn't deep enough — prefer the simpler, more
general change to the underlying mechanism over adding special cases, and name
that change.

### Conventions (instruction files)

Find the instruction files that govern the changed code: the user-level
`~/.claude/CLAUDE.md` and `~/.config/opencode/AGENTS.md` (often the same file:
read it once), the repo-root CLAUDE.md
and AGENTS.md, plus any CLAUDE.md, CLAUDE.local.md or AGENTS.md in a directory
that is an ancestor of a changed file (a directory's file only applies to files at
or below it). Read each one that exists, then check the diff for clear violations
of the rules they state.

Only flag a violation when you can quote the exact rule and the exact line
that breaks it — no style preferences, no vague "spirit of the doc"
inferences. In the finding, name the file path and quote the rule so the
report can cite it. If no instruction file applies, return nothing for this angle.

Cleanup, altitude, and conventions candidates use the same
`file`/`line`/`summary` shape; in `failure_scenario`, state the concrete
cost (what is duplicated, wasted, harder to maintain, or which instruction-file
rule is broken) instead of a crash. Correctness bugs always outrank cleanup,
altitude, and conventions findings when the output cap forces a cut.

## Output

Target **at least <half the level's usual cap, or of `--max-findings <n>` when
smaller, at least 1> findings**. If fewer genuine findings
exist, emit what you have — do not invent to hit the floor.

Return findings as a JSON array of at most <cap> objects (with `--max-findings all`:
one object for every finding that survives, no maximum):

```json
[
  {
    "file": "path/to/file.ext",
    "line": 123,
    "summary": "one-sentence statement of the bug",
    "failure_scenario": "concrete inputs/state → wrong output/crash"
  }
]
```

Ranked most-severe first. If more than <cap> survive, keep the <cap> most
severe. If nothing survives, return `[]`.

## Applying fixes (--fix)

Only when `--fix` was passed. After producing the findings list, apply the
findings to the working tree instead of stopping at the report: fix each one
directly — correctness bugs and reuse/simplification/efficiency cleanups alike.
Skip any finding whose fix would change intended behavior, require changes well
outside the reviewed diff, or that you judge to be a false positive — note the
skip rather than arguing with it. Finish with a brief summary of what was fixed
and what was skipped.

## Posting to GitHub or GitLab (--comment)

Only when `--comment` was passed. After producing the findings list, if the
review target is a GitHub PR, post each finding as an inline PR comment with
`gh api` (repos/{owner}/{repo}/pulls/{pr}/comments; include a suggestion block
only when it fully fixes the issue). If the target is a GitLab merge request,
post the findings as one general MR note with `glab mr note <mr> -m "<body>"`
(every finding with its file:line, the issue, and the suggested fix). If the
tool is not available in this session, print the findings instead. If the
target is not a PR or MR, print the findings and note that `--comment` was
ignored.

Without `--fix`, change no files. Without `--comment`, post nothing.
