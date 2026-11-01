var Y="## Phase 0 — Gather the diff\n\nRun `git diff @{upstream}...HEAD` (or `git diff main...HEAD` / `git diff HEAD~1`\nif there's no upstream) to get the unified diff under review. If there are\nuncommitted changes, or the range diff is empty, also run `git diff HEAD` and\ninclude the working-tree changes in scope — the review often runs before the\ncommit. If a PR number, branch name, or file path was passed as an argument,\nreview that target instead. Treat this diff as the review scope.\n",me=`Flag new code that re-implements something the codebase
already has — Grep shared/utility modules and files adjacent to the change,
and name the existing helper to call instead.
`,Z=`### Simplification

Flag unnecessary complexity the diff adds: redundant or derivable state,
copy-paste with slight variation, deep nesting, dead code left behind. Name
the simpler form that does the same job.
`,oe=`### Efficiency

Flag wasted work the diff introduces: redundant computation or repeated I/O,
independent operations run sequentially, blocking work added to startup or
hot paths. Also flag long-lived objects built from closures or captured
environments — they keep the entire enclosing scope alive for the object's
lifetime (a memory leak when that scope holds large values); prefer a
class/struct that copies only the fields it needs. Name the cheaper
alternative.
`,ce=`### Conventions (CLAUDE.md)

Find the CLAUDE.md files that govern the changed code: the user-level
~/.claude/CLAUDE.md, the repo-root CLAUDE.md, plus any CLAUDE.md or
CLAUDE.local.md in a directory that is an ancestor of a changed file (a
directory's CLAUDE.md only applies to files at or below it). Read each one
that exists, then check the diff for clear violations of the rules they state.

Only flag a violation when you can quote the exact rule and the exact line
that breaks it — no style preferences, no vague "spirit of the doc"
inferences. In the finding, name the CLAUDE.md path and quote the rule so the
report can cite it. If no CLAUDE.md applies, return nothing for this angle.
`,ne=`### Altitude

Check that each change fixes the root cause at the right depth rather than
patching a symptom with a fragile bandaid. Special cases layered on shared
infrastructure are a sign the fix isn't deep enough — prefer the simpler, more
general change to the underlying mechanism over adding special cases, and name
that change.
`;var us=p(()=>A(u({file:o(),line:Ue([E(),o()]).nullish(),summary:o(),failure_scenario:o().nullish()}))),ps=/^```json[ \t]*\r?\n([\s\S]*?)^```[ \t]*$/gim;function Ft(e){let n=Dre();if(n==="text"||n==="json")return e;return(e.trimStart().startsWith("[")?jt(e):void 0)??e.replace(ps,(r,h)=>jt(h)??r)}function jt(e){let n=us().safeParse(st(e.trim(),!1));if(!n.success)return;let s=n.data;if(s.length===0)return"No findings.";let r=s.map((y,w)=>{let b=y.line?`${y.file}:${y.line}`:y.file,k=y.failure_scenario?`

Failure scenario: ${y.failure_scenario}`:"",C=`${w+1}. `;return E0(`${C}\`${b}\`: ${y.summary}${k}`).replace(/\n(?=[^\n])/g,`
${" ".repeat(C.length)}`)}),h=s.length>1?", most severe first":"";return`${s.length} ${O(s.length,"finding")}${h}:

${r.join(`

`)}`}var ms=`### Angle A — line-by-line diff scan

Read every hunk in the diff, line by line. Then Read the enclosing function for
each hunk — bugs in unchanged lines of a touched function are in scope (the PR
re-exposes or fails to fix them). For every line ask: what input, state, timing,
or platform makes this line wrong? Look for inverted/wrong conditions,
off-by-one, null/undefined deref, missing \`await\`, falsy-zero checks,
wrong-variable copy-paste, error swallowed in catch, unescaped regex metachars.
`,fs=`### Angle B — removed-behavior auditor

For every line the diff DELETES or replaces, name the invariant or behavior it
enforced, then search the new code for where that invariant is re-established.
If you can't find it, that's a candidate: a removed guard, a dropped error
path, a narrowed validation, a deleted test that was covering a real case.
`,ys=`### Angle C — cross-file tracer

For each function the diff changes, find its callers (Grep for the symbol) and
check whether the change breaks any call site: a new precondition, a changed
return shape, a new exception, a timing/ordering dependency. Also check callees:
does a parallel change in the same PR make a call unsafe?
`,ws=`### Angle D — language-pitfall specialist

Scan for the classic pitfalls of the diff's language/framework — for example:
JS falsy-zero, \`==\` coercion, closure-captured loop var; Python mutable default
args, late-binding closures; Go nil-map write, range-var capture; SQL injection;
timezone/DST drift; float equality. Flag any instance the diff introduces.
`,vs=`### Angle E — wrapper/proxy correctness

When the PR adds or modifies a type that wraps another (cache, proxy, decorator,
adapter): check that every method routes to the wrapped instance and not back
through a registry/session/global — e.g. a caching provider holding a
\`delegate\` field that resolves IDs via \`session.get(...)\` instead of
\`delegate.get(...)\` will re-enter the cache or recurse. Also check that the
wrapper forwards all the methods the callers actually use.
`,Ke=`If the ${mt} tool is not available in your current tool set, do not error — perform each angle (and each verification) yourself, sequentially, in this context.`,Ht=`${ms}
${fs}
${ys}`,ks=`${Ht}
${ws}
${vs}`,Gt=`### Reuse

The angles above hunt for bugs; this one and the next two hunt for cleanup in
the changed code. ${me}`,le="Cleanup, altitude, and conventions candidates use the same\n`file`/`line`/`summary` shape; in `failure_scenario`, state the concrete\ncost (what is duplicated, wasted, harder to maintain, or which CLAUDE.md rule\nis broken) instead of a crash. Correctness bugs always outrank cleanup,\naltitude, and conventions findings when the output cap forces a cut.\n",Cs=`- **CONFIRMED** — can name the inputs/state that trigger it and the wrong
  output or crash. Quote the line.
- **PLAUSIBLE** — mechanism is real, trigger is uncertain (timing, env,
  config). State what would confirm it.
- **REFUTED** — factually wrong (code doesn't say that) or guarded elsewhere.
  Quote the line that proves it.`,_s=`**PLAUSIBLE by default** — do not refute a candidate for being "speculative" or
"depends on runtime state" when the state is realistic: concurrency races,
nil/undefined on a rare-but-reachable path (error handler, cold cache, missing
optional field), falsy-zero treated as missing, off-by-one on a boundary the
code does not exclude, retry storms / partial failures, regex/allowlist that
lost an anchor. These are PLAUSIBLE.

**REFUTED** only when constructible from the code: factually wrong (quote the
actual line); provably impossible (type/constant/invariant — show it); already
handled in this diff (cite the guard); or pure style with no observable effect.`,Wt=`## Phase 2 — Verify (1-vote, 3-state)

Dedup candidates that point at the same line/mechanism, keeping the one with
the most concrete failure scenario. For each remaining candidate, run **one
verifier** via the ${mt} tool: give it the diff, the relevant
file(s), and the candidate, and have it return exactly one of:

${Cs}

Keep candidates where the vote is CONFIRMED or PLAUSIBLE.
`,Es=`## Phase 2 — Verify (1-vote, recall-biased)

Dedup near-duplicates (same defect, same location, same reason → keep one). For
each remaining candidate, run **one verifier** via the ${mt} tool:
give it the diff, the relevant file(s), and the candidate; it returns exactly
one of **CONFIRMED / PLAUSIBLE / REFUTED**.

${_s}

Keep **CONFIRMED and PLAUSIBLE**. Drop REFUTED.
`,qt=`moved/extracted code that dropped a guard
or anchor; second-tier footguns (dataclass default evaluated once, \`hash()\`
non-determinism, lock-scope shrink, predicate methods with side effects);
setup/teardown asymmetry in tests; config defaults flipped.`,Ss=`## Phase 3 — Sweep for gaps

Run **one more finder** as a fresh reviewer who has the verified list. Re-read
the diff and enclosing functions looking ONLY for defects not already listed.
Do not re-derive or re-confirm anything already there — the job is gaps. Focus
on what the first pass tends to miss: ${qt}

Surface **up to 8 additional candidates**, each naming a defect not already on
the list. If nothing new, return an empty sweep — do not pad.
`;function G(e){return e==="all"?"all findings":`≤${e} ${O(e,"finding")}`}function ge(e){return e==="all"?"**every finding**":`at most **${e} ${O(e,"finding")}**`}var Yt=(e)=>`## Output

Return findings as a JSON array ${e==="all"?"with one object for every finding that survives, no maximum":`of at most ${e} ${O(e,"object")}`}:

\`\`\`json
[
  {
    "file": "path/to/file.ext",
    "line": 123,
    "summary": "one-sentence statement of the bug",
    "failure_scenario": "concrete inputs/state → wrong output/crash"
  }
]
\`\`\`

Ranked most-severe first. ${e==="all"?"Keep every finding that survives. There is no maximum.":`If more than ${e} survive, keep the ${e} most
severe.`} If nothing survives verification, return \`[]\`. Do not call the
${cO} tool even if it is available - this review's
output contract is the JSON block above.
`,zt=(e)=>`## Output

Call the ${cO} tool once to report this review's results
with \`{level, findings}\`. \`findings\` is ${e==="all"?"every finding that survives, no maximum,":`at most ${e} ${O(e,"entry","entries")}`} ranked
most-severe first; each entry has \`file\`, \`line\`, \`summary\`,
\`short_summary\` — the claim compressed to ≤60 characters, no rationale
or consequence clause — \`failure_scenario\`, and \`category\` — a short kebab-case slug for the angle
that produced it (\`correctness\`, \`simplification\`, \`efficiency\`,
\`reuse\`, \`altitude\`, \`conventions\`, or a more specific slug like
\`test-coverage\` when one fits better) — plus \`verdict\` when a verify pass
produced one. ${e==="all"?"Keep every finding that survives. There is no maximum.":`If more than ${e} survive, keep the ${e} most severe.`} If
nothing survives verification, call it with an empty array. Do not also print
the findings as text, and do not create or publish an artifact of the review -
the tool call is the report.
`,Vt=(e,n=4)=>`\`low effort → 1 diff pass → no verify → ${G(n)}\`

## Turn 1 — read

One tool call: read the unified diff (\`git diff @{upstream}...HEAD; git diff HEAD\`
to cover both committed and uncommitted changes, or \`git diff main...HEAD\` /
the target passed as an argument). Skip test/fixture
hunks (\`test/\`, \`spec/\`, \`__tests__/\`, \`*_test.*\`, \`*.test.*\`,
\`fixtures/\`, \`testdata/\`) — test-file changes are not reviewed at this level.
No subagents, no full-file reads.

## Turn 2 — findings

Flag runtime-correctness bugs visible from the hunk alone: inverted/wrong
condition, off-by-one, null/undefined deref where adjacent lines show the value
can be absent, removed guard, falsy-zero check, missing \`await\`,
wrong-variable copy-paste, error swallowed in a catch that should propagate.
Also flag — still from the hunk alone — new code that duplicates an existing
helper visible in the diff context, and dead code the diff leaves behind.

Do **not** flag style, naming, perf, missing tests, or anything outside the
hunk.

${e?`Report ${ge(n)}, most-severe first, in one
${cO} call with \`{level, findings}\` — each entry has
\`file\`, \`line\`, \`summary\`, \`short_summary\` (≤60 characters), and
\`failure_scenario\`. If nothing qualifies, call it with an empty findings
array. Do not also print the findings as text.
`:`Output ${ge(n)}, most-severe first, one line each:
\`path/to/file.ext:123 — what's wrong and the concrete failure\`. If nothing
qualifies, output exactly \`(none)\`. Do not call the
${cO} tool even if it is available.
`}`,Kt=(e)=>`\`low effort → 1 diff pass → no verify → ≥min(files,4) findings\`

## Turn 1 — read

One tool call: read the unified diff (\`git diff @{upstream}...HEAD; git diff HEAD\`
to cover both committed and uncommitted changes, or \`git diff main...HEAD\` /
the target passed as an argument). Skip test/fixture
hunks (\`test/\`, \`spec/\`, \`__tests__/\`, \`*_test.*\`, \`*.test.*\`,
\`fixtures/\`, \`testdata/\`) — test-file changes are not reviewed at this level.
No subagents, no full-file reads.

## Turn 2 — findings

Flag runtime-correctness bugs visible from the hunk alone: inverted/wrong
condition, off-by-one, null/undefined deref where adjacent lines show the value
can be absent, removed guard, falsy-zero check, missing \`await\`,
wrong-variable copy-paste, error swallowed in a catch that should propagate.
Also flag — still from the hunk alone — new code that duplicates an existing
helper visible in the diff context, and dead code the diff leaves behind.

Do **not** flag style, naming, perf, missing tests, or anything outside the
hunk.

${e?`Target **min(files_changed, 4) findings**, most-severe first, reported
in one ${cO} call with \`{level, findings}\` — each
entry has \`file\`, \`line\`, \`summary\`, \`short_summary\` (≤60 characters),
and \`failure_scenario\`. If you have fewer, do one more pass focused on the
largest changed file and on any **removed** code blocks. Call it with an
empty findings array only if the diff is trivially correct after that pass.
Do not also print the findings as text.
`:`Target **min(files_changed, 4) findings**, most-severe first, one
line each: \`path/to/file.ext:123 — what's wrong and the concrete failure\`.
If you have fewer, do one more pass focused on the largest changed file
and on any **removed** code blocks. Output \`(none)\` only if the diff is
trivially correct after that pass.
`}`,Se=`${Ht}
${Gt}
${Z}
${oe}
${ne}
${ce}`,Ts=`The ${mt} tool isn't available in this context, so the usual
multi-agent fan-out and subagent verify pass can't run. Work through every
angle below yourself, in this same context, in one pass — do not skip angles
for lack of fan-out. Re-check each candidate against the diff before keeping
it; drop anything you can't back up with a concrete failure scenario.
`,As=`
State clearly in your summary that this was a single-pass review done without
the ${mt} tool, not the full multi-agent fan-out, so whoever reads
it isn't misled about what actually ran.
`;function Xe({tag:e,leadIn:n,angleCount:s,angles:r,cap:h,output:y,sweepFocus:w}){let b=w?`
## Phase 3 — Sweep for gaps

Take one more pass yourself (same context, no subagent) as a fresh reviewer
who has the deduplicated list. Re-read the diff and enclosing functions
looking ONLY for defects not already listed: ${w}
`:"";return`\`${e}\`

${n}

${Ts}
${Y}## Phase 1 — Find candidates (${s} angles, single pass)

Work through **${s} angles** yourself, in sequence, in this same
context — do not spawn subagents. Each surfaces candidate findings with
\`file\`, \`line\`, a one-line \`summary\`, and a concrete \`failure_scenario\`.

${r}
${le}
## Phase 2 — Dedup and self-check (no subagent verify)

Dedup near-duplicates (same defect, same location, same reason → keep one).
Re-check each remaining candidate yourself against the diff before keeping it.
${b}
${y(h)}${As}`}var Xt=(e,n=!0,s=8)=>{if(!n)return Xe({tag:`medium effort → ${mt} tool unavailable → single-pass inline → ${G(s)}`,leadIn:`You are reviewing for **precision** at medium effort: every finding you surface
should be one a maintainer would act on.`,angleCount:8,angles:Se,cap:s,output:e});return`\`medium effort → 3+5 angles × 6 candidates → 1-vote verify → ${G(s)}\`

You are reviewing for **precision** at medium effort: every finding you surface
should be one a maintainer would act on.

${Y}
## Phase 1 — Find candidates (3 correctness angles + 3 cleanup angles + 1 altitude angle + 1 conventions angle, up to 6 each)

Run **8 independent finder angles** via the ${mt} tool. Each
surfaces **up to 6 candidate findings** with \`file\`, \`line\`, a one-line
\`summary\`, and a concrete \`failure_scenario\`. ${Ke}

${Se}
${le}
Pass every candidate with a nameable failure scenario through — finders that
silently drop half-believed candidates bypass the verify step and are the
dominant cause of misses.

${Wt}
${e(s)}`},Jt=(e,n=!0,s=10)=>{if(!n)return Xe({tag:`high effort → ${mt} tool unavailable → single-pass inline → ${G(s)}`,leadIn:`You are reviewing for **recall** at high effort: catch every real bug a careful
reviewer would catch in one sitting. At this level, catching real bugs matters
more than avoiding false positives. Err on the side of surfacing.`,angleCount:8,angles:Se,cap:s,output:e});return`\`high effort → 3+5 angles × 6 candidates → 1-vote verify (recall-biased) → ${G(s)}\`

You are reviewing for **recall** at high effort: catch every real bug a careful
reviewer would catch in one sitting. At this level, catching real bugs matters
more than avoiding false positives. Err on the side of surfacing.

${Y}
## Phase 1 — Find candidates (3 correctness angles + 3 cleanup angles + 1 altitude angle + 1 conventions angle, up to 6 each)

Run **8 independent finder angles** via the ${mt} tool. Each
surfaces **up to 6 candidate findings** with \`file\`, \`line\`, a one-line
\`summary\`, and a concrete \`failure_scenario\`. ${Ke}

${Se}
${le}
Pass every candidate with a nameable failure scenario through — finders that
silently drop half-believed candidates bypass the verify step and are the
dominant cause of misses.

${Es}
${e(s)}`},Bt=`${ks}
${Gt}
${Z}
${oe}
${ne}
${ce}`,Qt=(e)=>(n,s=!0,r=15)=>{if(!s)return Xe({tag:`${e} effort → ${mt} tool unavailable → single-pass inline → ${G(r)}`,leadIn:`You are reviewing for **recall** at ${e==="max"?"maximum":"extra-high"} effort: catch every real bug. At
this level, catching real bugs matters more than avoiding false positives — a
missed bug ships. Err on the side of surfacing.`,angleCount:10,angles:Bt,cap:r,output:n,sweepFocus:qt});return`\`${e} effort → 5+5 angles × 8 candidates → 1-vote verify → sweep → ${G(r)}\`

You are reviewing for **recall** at ${e==="max"?"maximum":"extra-high"} effort: catch every real bug. At
this level, catching real bugs matters more than avoiding false positives — a
missed bug ships. Err on the side of surfacing.

${Y}
## Phase 1 — Find candidates (5 correctness angles + 3 cleanup angles + 1 altitude angle + 1 conventions angle, up to 8 each)

Run **10 independent finder angles** via the ${mt} tool. Each
surfaces **up to 8 candidate findings**. Do NOT let one angle's conclusions
suppress another's — if two angles flag the same line for different reasons,
record both. ${Ke}

${Bt}
${le}
${Wt}
This is recall mode — a single non-REFUTED vote carries the finding. Do NOT
drop on uncertainty.

${Ss}
${n(r)}`},Zt=Qt("xhigh"),to=Qt("max");var oo=`### Reuse

The angles above hunt for bugs; this one and the next two hunt for cleanup in
the changed code. Flag new code that re-implements something the codebase
already has — Grep shared/utility modules and files adjacent to the change,
and name the existing helper to call instead.
`,no=(e,n=8)=>{let s=n==="all"?4:Math.min(4,n);return`\`low effort → 1 diff pass → no verify → ${G(n)}\`

## Turn 1 — read

One tool call: read the unified diff (\`git diff @{upstream}...HEAD; git diff HEAD\`
to cover both committed and uncommitted changes, or \`git diff main...HEAD\` /
the target passed as an argument). No subagents, no full-file reads.

## Turn 2 — findings

Flag runtime-correctness bugs visible from the hunk alone: inverted/wrong
condition, off-by-one, null/undefined deref where adjacent lines show the value
can be absent, removed guard, falsy-zero check, missing \`await\`,
wrong-variable copy-paste, error swallowed in a catch that should propagate.
Also flag — still from the hunk alone — new code that duplicates an existing
helper visible in the diff context, and dead code the diff leaves behind.

Do **not** flag style, naming, perf, missing tests, or anything outside the
hunk.

${e?`Report ${ge(n)}, most-severe first, in one
${cO} call with \`{level, findings}\` — each entry has
\`file\`, \`line\`, \`summary\`, \`short_summary\` (≤60 characters), and
\`failure_scenario\`.
Target at least min(files_changed, ${s}) findings — if you see fewer, widen to other hunks in the same diff before stopping. If fewer than ${s} genuine findings exist, report what you have. Do not also print the findings as text.
`:`Output ${ge(n)}, most-severe first, one line each:
\`path/to/file.ext:123 — what's wrong and the concrete failure\`.
Target at least min(files_changed, ${s}) findings — if you see fewer, widen to other hunks in the same diff before stopping. If fewer than ${s} genuine findings exist, emit what you have.
`}`},so=(e,n)=>(s)=>{let r=Math.max(1,Math.floor((s==="all"?n:Math.min(s,n))/2));return e(s).replace(`## Output
`,`## Output

Target **at least ${r} ${O(r,"finding")}**. If fewer genuine findings exist, emit what you have — do not invent to hit the floor.
`).replace(/nothing survives verification/g,"nothing survives")},ao=`### Angle A — line-by-line diff scan

Read every hunk in the diff, line by line. Then Read the enclosing function for
each hunk — bugs in unchanged lines of a touched function are in scope (the PR
re-exposes or fails to fix them). For every line ask: what input, state, timing,
or platform makes this line wrong? Look for inverted/wrong conditions,
off-by-one, null/undefined deref, missing \`await\`, falsy-zero checks,
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
`,ro=(e,n,s)=>(r,h=s)=>`\`${e} → ${G(h)}\`

${n}

${Y}
## Phase 1 — Find candidates (3 correctness angles + 3 cleanup angles + 1 altitude angle + 1 conventions angle, up to 6 each)

Run **8 independent finder angles** in sequence yourself, in THIS context — do NOT spawn subagents for them. Each
surfaces **up to 6 candidate findings** with \`file\`, \`line\`, a one-line
\`summary\`, and a concrete \`failure_scenario\`.

${ao}
${oo}
${Z}
${oe}
${ne}
${ce}
${le}
Pass every candidate with a nameable failure scenario through — finders that
silently drop half-believed candidates are the dominant cause of misses.

## Phase 2 — Dedup only (no verify)

Pool all candidates. Dedup near-duplicates only (same defect, same location, same reason → keep one). Do NOT run verifiers; do NOT re-judge. Sort by severity.

${so(r,s)(h)}`,lo=ro("medium effort → 8 inline angles → dedup (no verify)",`You are reviewing for **correctness bugs**: surface every plausible bug. At this
level, catching real bugs matters more than avoiding false positives — err on
the side of surfacing.`,8),co=ro("high effort → 8 inline angles → dedup (no verify)",`You are reviewing for **recall** at high effort: catch every real bug a careful
reviewer would catch in one sitting. At this level, catching real bugs matters
more than avoiding false positives. Err on the side of surfacing.`,10),xs=(e,n=15)=>`\`xhigh effort → 10 inline angles → dedup (no verify) → sweep → ${G(n)}\`

You are reviewing for **recall** at extra-high effort: catch every real bug. At
this level, catching real bugs matters more than avoiding false positives — a
missed bug ships. Err on the side of surfacing.

${Y}
## Phase 1 — Find candidates (5 correctness angles + 3 cleanup angles + 1 altitude angle + 1 conventions angle, up to 8 each)

Run **10 independent finder angles** in sequence yourself, in THIS context — do NOT spawn subagents for them. Each
surfaces **up to 8 candidate findings**. Do NOT let one angle's conclusions
suppress another's — if two angles flag the same line for different reasons,
record both.

${ao}
### Angle D — language-pitfall specialist

Scan for the classic pitfalls of the diff's language/framework — for example:
JS falsy-zero, \`==\` coercion, closure-captured loop var; Python mutable default
args, late-binding closures; Go nil-map write, range-var capture; SQL injection;
timezone/DST drift; float equality. Flag any instance the diff introduces.

### Angle E — wrapper/proxy correctness

When the PR adds or modifies a type that wraps another (cache, proxy, decorator,
adapter): check that every method routes to the wrapped instance and not back
through a registry/session/global — e.g. a caching provider holding a
\`delegate\` field that resolves IDs via \`session.get(...)\` instead of
\`delegate.get(...)\` will re-enter the cache or recurse. Also check that the
wrapper forwards all the methods the callers actually use.

${oo}
${Z}
${oe}
${ne}
${ce}
${le}
## Phase 2 — Dedup only (no verify)

Pool all candidates. Dedup near-duplicates only (same defect, same location, same reason → keep one). Do NOT run verifiers; do NOT re-judge. Sort by severity. Do NOT drop on uncertainty.

## Phase 3 — Sweep for gaps

Take one more pass (same context — no subagent) as a fresh reviewer who has the deduplicated list. Re-read
the diff and enclosing functions looking ONLY for defects not already listed.
Do not re-derive or re-confirm anything already there — the job is gaps. Focus
on what the first pass tends to miss: moved/extracted code that dropped a guard
or anchor; second-tier footguns (dataclass default evaluated once, \`hash()\`
non-determinism, lock-scope shrink, predicate methods with side effects);
setup/teardown asymmetry in tests; config defaults flipped.

Surface **up to 8 additional candidates**, each naming a defect not already on
the list. If nothing new, return nothing from this phase — do not pad.

${so(e,15)(n)}`,ho=xs;var Je=(e=15)=>`\`minimal prompt → single careful diff pass → ${G(e)}\`

You are reviewing a pull request for real bugs. Run \`git diff @{upstream}...HEAD\` (or \`git diff main...HEAD\` / \`git diff HEAD~1\`
if there's no upstream) to get the unified diff under review. If there are
uncommitted changes, or the range diff is empty, also run \`git diff HEAD\` and
include the working-tree changes in scope — the review often runs before the
commit. If a PR number, branch name, or file path was passed as an argument,
review that target instead. Treat this diff as the review scope.

Review the diff as a careful senior engineer would: read every hunk, open the surrounding files for context as needed (Read, Grep, git log/blame/show), and hunt for correctness issues — wrong or inverted conditions, off-by-one, null/undefined dereference, missing \`await\`, dropped error handling, removed guards or validations, broken callers of changed functions, races. Prefer real failure modes over style; every finding needs a concrete scenario in which the code misbehaves.

When you are done, submit ${e==="all"?"every finding":`at most ${e} ${O(e,"finding")}`} via the ${cO} tool, filling its fields as defined — for each: the file path and start line, a severity, and a comment that states the issue and the concrete scenario in which the code misbehaves. Quality over quantity: include everything you genuinely believe is a real issue, and nothing you don't.

After the tool call, also restate the findings in your final reply — one line each, \`file:line — summary\` — so they stay visible in sessions that do not render tool output.
`,zl=Je();function Is(e){return Object.hasOwn(de,e)}function Ae(e){let n=e?Qe(Lt(e)):void 0;return n&&Is(n)?n:"default"}var Ps={cell:"low",modelEffort:"typed",finderBudgetHint:!1},F=(e)=>({cell:e,modelEffort:"typed",finderBudgetHint:!1}),Rs=new Set(["claude-opus-4-8","claude-opus-5"]),Ls={"claude-sonnet-5":"sonnet5","claude-opus-4-8":"hc10"},po={low:Ps,medium:{...F("o48-high-v1"),measuredExternal:!0},high:{...F("o48-high-v1"),measuredExternal:!0},xhigh:{...F("o48-xhigh-v1"),measuredExternal:!0},max:F("max")},Os={...po,medium:{...F("o5-bmin"),measuredExternal:!0},high:{...F("o5-bmin"),measuredExternal:!0}},de={default:po,"claude-sonnet-5":{low:{cell:"low-sonnet5",modelEffort:"medium",finderBudgetHint:!1},medium:F("medium"),high:{...F("high"),finderBudgetHint:!0},xhigh:{...F("xhigh"),finderBudgetHint:!0},max:{...F("max"),finderBudgetHint:!0}},"claude-opus-4-8":{low:{...F("o48-low-v1"),measuredExternal:!0},medium:{...F("o48-med-v1"),measuredExternal:!0},high:{...F("o48-high-v1"),measuredExternal:!0},xhigh:{...F("o48-xhigh-v1"),measuredExternal:!0},max:F("max")},"claude-opus-5":Os};for(let e of Object.values(de)){for(let n of Object.values(e))Object.freeze(n);Object.freeze(e)}Object.freeze(de);function tt(e,n){let s=de[e][n];return s.modelEffort==="typed"?n:s.modelEffort}function Ds(e,n,s=!0,r=!1,h){switch(e){case"low":return Vt(r,h);case"low-sonnet5":return Kt(r);case"medium":return Xt(n,s,h);case"high":return Jt(n,s,h);case"xhigh":return Zt(n,s,h);case"max":return to(n,s,h);case"o48-low-v1":return no(r,h);case"o48-med-v1":return lo(n,h);case"o48-high-v1":return co(n,h);case"o48-xhigh-v1":return ho(n,h);case"o5-bmin":return Je(h)}}var Ns=new Set(["remote","remote_desktop","remote_mobile","remote_trigger"]);function ot(e){if(e.options?.isSkillPreload||e.agentId!==void 0)return;let n=Dre();if(n==="text"||n==="json")return;if(!e.options?.tools?.some((h)=>Pt(h,cO)))return;if(i.CLAUDE_CODE_REPORT_FINDINGS)return"spawn";let s=e.options?.messageClientPlatform,r=i.CLAUDE_CODE_ENTRYPOINT;return(s==="desktop_app"||s==="web_claude_ai")&&!_h()&&(vS()||Boolean(i.CLAUDE_CODE_REMOTE)&&r!==void 0&&Ns.has(r))&&bn("tengu_purring_micali",!0)?"message":void 0}var $s=`

## Posting to GitHub (--comment)

The \`--comment\` flag was passed. After producing the findings list, if the
review target is a GitHub PR, post each finding as an inline PR comment via
\`mcp__github_inline_comment__create_inline_comment\` (one call per finding;
include a suggestion block only when it fully fixes the issue). If that tool
is not available in this session, fall back to \`gh api\` (repos/{owner}/{repo}/pulls/{pr}/comments)
or print the findings instead. If the target is not a PR, print the findings
to the terminal and note that \`--comment\` was ignored.
`;function Ms(e){let n=Ohs(e),s=Mhs(e),r=s?` -R ${s}`:"";return`

## Posting to GitLab (--comment)

The \`--comment\` flag was passed. After producing the findings list, if the
review target is a GitLab merge request, post the findings as one general MR
note via \`${`glab mr note${n?` ${n}`:""}${r} -m "<body>"`}\`${s?"":" from inside that project's checkout"}
(every finding with its file:line, the issue, and the suggested fix). glab has no single verb for line-anchored
comments; those require \`glab api projects/:id/merge_requests/:iid/discussions\`,
so post the general note unless the user asks for inline threads. If glab is
not available in this session, print the findings instead. If the target is
not an MR, print the findings to the terminal and note that \`--comment\` was
ignored.
`}var mo=`call ${cO} again with the same findings, each
carrying an \`outcome\`: \`fixed\`, \`no_change_needed\` (the finding was wrong or
already handled), or \`skipped\` (real but not applied). Do not repeat the
findings as text`,Us=`

## If findings are fixed later

Whenever reported findings get fixed later in this session - the user asks you
to fix them, or later work fixes them incidentally - you MUST ${mo}.
Make that call immediately after the fixes land, before any prose summary; the
host UI's per-finding status updates only from it, and without it the findings
stay marked unresolved.
`,js=`After the call, give one line per finding with its outcome: this session can
also be read from a client that does not show the tool call.
`;function Fs(e){return`

## Applying fixes (--fix)

The \`--fix\` flag was passed. After producing the findings list, apply the
findings to the working tree instead of stopping at the report: fix each one
directly — correctness bugs and reuse/simplification/efficiency cleanups alike.
Skip any finding whose fix would change intended behavior, require changes well
outside the reviewed diff, or that you judge to be a false positive — note the
skip rather than arguing with it. ${e?`Then ${mo}; after the call, give one line per skipped finding saying why.`:`Finish with a brief summary of what was fixed
and what was skipped.`}
`}var Bs=`

## After the review

After the findings are reported (and applied, when --fix was passed): if \`/${Ipe}\` has not run this session and the diff has a runtime surface (not test-only or docs-only per the pre-ship exemptions), invoke \`/${Ipe}\` now — this review checks that the diff reads right; \`/${Ipe}\` checks that it runs right.
`;async function Gs(e){if(e.options?.isSkillPreload)return"";if(!Vno())return"";if(!dmi(e.getProactivityLevel()))return"";let n=e.options?.tools;if(n&&!n.some((r)=>Pt(r,qr)))return"";return(await smn(go(),e.storageV5)).some((r)=>r.name===Ipe)?Bs:""}var he=Ei,Ws=new RegExp(`^(${he.map((e)=>e.slice(0,3)).join("|")})[a-z]*$`,"i"),uo=/(^|\s)--max-findings(?:(?:\s*=\s*|\s+)(?!--)(\S+)|\s*=\S*)?(?=\s|$)/g;function et(e){let[n="",...s]=e;return[n.replaceAll("`","").replace(/^#/,""),...s].filter(Boolean).join(" ")}function Te(e){let{rawFirstToken:n}=Ve(e,[]),{flags:s,rest:r}=Ve(e.replace(uo,"$1"),["comment","fix","post","no-post"]),h=s.has("comment"),y=s.has("fix"),w=s.has("post"),b=[...e.matchAll(uo)].at(-1),k=b?(b[2]??"").toLowerCase():void 0,C=k!==void 0&&/^\d+$/.test(k)?Number(k):0,S=k==="all"||k==="default"?k:Number.isSafeInteger(C)&&C>0?C:void 0,T=k!==void 0&&S===void 0,R=r.split(/\s+/).filter(Boolean),L=R[0]??"";if(n.toLowerCase()==="ultra")return{explicit:void 0,target:et(R.slice(1)),comment:h,fix:y,post:w,unrecognizedLevel:void 0,ultraFallback:!0,maxFindings:S,maxFindingsIgnored:T};let x=L.toLowerCase()==="ultra"?void 0:IH(L);if(x!==void 0)return{explicit:x,target:et(R.slice(1)),comment:h,fix:y,post:w,unrecognizedLevel:void 0,ultraFallback:!1,maxFindings:S,maxFindingsIgnored:T};let D=Ws.test(L);return{explicit:void 0,target:et(R),comment:h,fix:y,post:w,unrecognizedLevel:D?L:void 0,ultraFallback:!1,maxFindings:S,maxFindingsIgnored:T}}function qs(){let e=ue().codeReviewLastEffort;return e!==void 0&&yE(e)?e:void 0}function Ys(e,n){Pe((s)=>s.codeReviewLastEffort===e?s:{...s,codeReviewLastEffort:e},n)}function nt({explicit:e,ultraFallback:n},s){if(s?.options?.isSkillPreload)return;return e===void 0&&!n?qs():void 0}var xe=32;function fo({maxFindings:e},n,s,r){let h=ue().codeReviewLastMaxFindings,y=e===void 0&&n!=="low-sonnet5"&&!r?.options?.isSkillPreload&&(h==="all"||typeof h==="number"&&Number.isSafeInteger(h)&&h>0),w=y?h:e==="default"?void 0:e,k=n==="low-sonnet5"?void 0:(s||n==="o5-bmin")&&(w==="all"||w!==void 0&&w>xe)?xe:w;return{asked:w,stated:k,reused:y}}function yo({asked:e,stated:n,reused:s,ignored:r},h){let y=[];if(r)y.push(`\`--max-findings\` was ignored: type a whole number above zero, \`all\`, or \`default\` after it.${s?"":" Using the usual limit."}`);let w=`This review reports at most ${xe} findings, so the limit is ${xe} here.`;if(e!==void 0&&n===void 0)y.push("`--max-findings` does not change this review: low effort here has no maximum.");else if(n!==e)y.push(s?`${h==="you"?"You":"The user"} set \`--max-findings ${e}\` last time. ${w}`:w);else if(s&&e!==void 0)y.push(e==="all"?`Reporting every finding, as ${h} chose last time with \`--max-findings all\`.`:`Reporting up to ${e} ${O(e,"finding")}, the limit ${h} set last time with \`--max-findings\`.`);return y.join(" ")}function zs(){let e=Uvt()?`; ultra: deep multi-agent review in the cloud${UF()?"":" (requires claude.ai account access)"}`:"",n=Uvt()?" For ultra on a GitHub.com PR target, --post asks to post the finished review’s findings to the PR as a single comment from the user’s GitHub account (not a review; the launch dialog still confirms in interactive sessions, while non-interactive mode posts on the flag alone) and --no-post hides that option.":"";return`Review the current diff, or a PR number/branch/path target, for correctness bugs (plus reuse/simplification/efficiency cleanups where the model's recipe covers them) at the effort level given first, before any target (${he.join("|")}: from few high-confidence findings to many, some uncertain${e}); with no level given, it reuses the level you typed last. Pass --comment to post findings as inline PR comments, or --fix to apply them to the working tree after the review. Pass --max-findings <n> to report up to n findings, or --max-findings all for every finding. The choice stays until you pass --max-findings default.${n}`}function Vs(){return Uvt()?"Use the max or ultra level only when the user, or instructions such as CLAUDE.md or a skill, explicitly ask for one, never on your own judgment: they are the costliest reviews and start many reviewer subagents.":"Use the max level only when the user, or instructions such as CLAUDE.md or a skill, explicitly ask for it, never on your own judgment: it is the costliest level and starts many reviewer subagents."}function Ks(){return`[${Uvt()?`${he.join("|")}|ultra`:he.join("|")}] [--fix] [--comment] [<pr#>|<branch>|<path>] [--max-findings <n>|all]`}async function Xs(e,n){let s=Te(e),{explicit:r,target:h,comment:y,fix:w,post:b,unrecognizedLevel:k,ultraFallback:C}=s,S=nt(s,n),T=wo(s,n),R=n.options?mc(n):void 0,L=Ae(R),x=n.options?.isSkillPreload&&Rs.has(L)?"default":L,D=de[x][T],B=ot(n),j=B!==void 0,W=!j,P=j?zt:Yt,N=fo(s,D.cell,j,n),q=D.cell==="o5-bmin",Re=B==="spawn"&&!D.measuredExternal?await Gs(n):"",se=Ze(h," "),we=Ihs(se),ve=Zs({ultraFallback:C,fix:w,post:b,comment:y,gitlabTarget:we,unrecognizedLevel:k,lastUsed:S,level:T,willRunAsFork:W,context:n}),re=yo({...N,ignored:s.maxFindingsIgnored},"the user"),H=N.reused?"passing `--max-findings default` goes back to the usual limit":void 0,be=!re?"":W?`(${re} Open your report with one short line telling the user this${H?`, and that ${H}`:""}; that opening line reaches them with the findings.)

`:`(${re} Tell the user this in one short line as you begin${H?`, including that ${H}`:""}.)

`,pe=Ee(n),ct=null,Oe={text:""};if(!n.options?.isSkillPreload){if(pe)Oe=await Js(R,T,h,ct);let De=r??S;a("tengu_code_review_routed",{effort_level:c(T),effort_source:c(r!==void 0?"explicit":S!==void 0?"last_used":C?"ultra_fallback":"session"),routed_to_workflow:!1,uses_report_findings_tool:j,has_fix:w,has_comment:y,has_target:h.length>0,is_ultra_fallback:C,low_variant:T==="low"?c(Ls[x]??"default"):void 0,model_family:c(x),finder_budget:Oe.budget,agent_tool_available:pe,threaded_effort:De!==void 0?c(tt(x,De)):void 0,findings_limit:c(N.stated===void 0?"default":N.stated==="all"?"all":"number"),findings_limit_value:typeof N.stated==="number"?N.stated:void 0})}let ht=n.options?.isSkillPreload||n.agentId!==void 0||C||W||D.measuredExternal?null:h1s(n.storageV5,n.credentials),fn=ht!==null?`

After you finish the review, end your response with this exact line on its own:
${ht}`:"",gn=h?`Review target: \`${h}\`

`:"",yn=ct?.preamble??"";return[{type:"text",text:`${ve}${be}${gn}${yn}${Oe.text}${Ds(D.cell,P,pe,j,N.stated)}${y?we?Ms(se):$s:""}${w?Fs(j):""}${j&&!q?Us:""}${B==="message"&&!q?js:""}${Re}${fn}`}]}async function Js(e,n,s,r){if(!de[Ae(e)][n].finderBudgetHint)return{text:""};let h=r?s?void 0:await r.countDiffLines():await Qs(s);if(h===void 0)return{text:""};let y=Math.max(2,Math.min(8,Math.ceil(h/150)));if(!s&&!r)return{text:`The committed diff (@{upstream}...HEAD) is about ${h} lines. Uncommitted changes aren't counted here, so treat this as a floor — start with about ${y} finder subagents (min 2, max 8) and scale up if Phase 0 finds additional working-tree scope.

`,budget:y};return{text:`This diff is about ${h} lines. Spawn about ${y} finder subagents (min 2, max 8) — scale your investigation depth to the diff size rather than using a fixed large fleet.

`,budget:y}}async function Qs(e){let n;if(!e)n="@{upstream}...HEAD";else if(e.length<=256&&/^[@\w][@\w./~^-]*\.\.\.?[@\w][@\w./~^-]*$/.test(e))n=e;else return;try{let{stdout:s,code:r}=await it(Mt(),["-c","core.hooksPath=/dev/null","-c","core.fsmonitor=","-c","core.askPass=","diff","--no-ext-diff","--no-textconv","--numstat","--end-of-options",n,"--"],{timeout:5000,useCwd:!0,env:{...Hs(),[["SELF_HOSTED","RUNNER_POOL_SECRET"].join("_")]:void 0,[["SELF_HOSTED","RUNNER_ENVIRONMENT_SECRET"].join("_")]:void 0}});if(r!==0)return;let h=0;for(let y of s.split(`
`)){let w=y.match(/^(\d+)\t(\d+)\t/);if(w)h+=Number(w[1])+Number(w[2])}return h>0?h:void 0}catch{return}}function wo(e,n){let{explicit:s,ultraFallback:r}=e,h=r?"max":s??nt(e,n),y=n.options?mc(n):void 0,w=y?aS(y,Mh(n),{turnEffort:h??w_(n.permissionLayers)})??h:h??Mh(n);return w===void 0?"medium":WN(w)}function Zs({ultraFallback:e,fix:n,post:s,comment:r,gitlabTarget:h,unrecognizedLevel:y,lastUsed:w,level:b,willRunAsFork:k,context:C}){let S=r?h?"when the target is a GitLab merge request, your `--comment` is what posts the findings as one general MR note via glab":"when the target is a GitHub PR, your `--comment` is what posts the findings as inline PR comments":h?"this local review will not post to GitLab; `--comment` is the flag that posts local findings as a general MR note":"this local review will not post to GitHub; `--comment` is the flag that posts local findings as inline PR comments",T=(x)=>s?`${x}(The typed \`--post\` applies only to the \`/code-review ultra\` cloud review and was ignored — ${S}. Tell the user this in one short line.)

`:x;if(e){if(!UF()){if(n)return T(`(Running a local ${b}-effort review and applying its findings.)

`);if(Uvt()){if(C.options?.isNonInteractiveSession){let D=qer();if(D)return T(`(${D} Falling back to a local ${b}-effort review.)

`)}return T(`(ultra (cloud review) requires claude.ai account access this session doesn't have — see https://code.claude.com/docs/en/ultrareview. Falling back to a local ${b}-effort review.)

`)}return T(`(ultra (cloud review) isn't available in this environment — see https://code.claude.com/docs/en/ultrareview. Falling back to a local ${b}-effort review.)

`)}let x=C.options?.commands?.some((D)=>D.name==="ultrareview"&&Ud(D))??!1;if(n)return T(x?`(Claude can't launch the cloud review directly — type \`/code-review ultra --fix\` to review in the cloud and apply the findings locally when it completes. Running a local ${b}-effort review and applying its findings for now.)

`:`(Running a local ${b}-effort review and applying its findings.)

`);return T(x?`(Claude can't launch the cloud review directly — type \`/code-review ultra\` to run it. Falling back to a local ${b}-effort review for now.)

`:`(Claude can't launch the cloud review directly — the user can run \`claude ultrareview\` from a terminal to start it. Falling back to a local ${b}-effort review for now.)

`)}let R="typing a level (for example `/code-review high`) changes it",L=(x)=>k?`(${x} Open your report with one short line telling the user this, and that ${R}; that opening line reaches them with the findings.)

`:`(${x} Tell the user this in one short line as you begin, including that ${R}.)

`;if(y!==void 0){let x=`Ignoring unrecognized effort "${y}"; valid: ${he.join(", ")}. Using ${b}${w===b?", the level the user typed last time":""}.`;return T(w!==void 0?L(x):`(${x})

`)}if(w!==void 0){let x=`reusing ${w}, the level the user typed last time${b!==w?`; running at ${b} here`:""}`;return T(L(`No effort level given — ${x}.`))}return T("")}function vo(){ol({name:B6,aliases:["review"],menuDescription:"Review the current diff or a PR for bugs and cleanups",subcommands:{ultra:"ultrareview"},description:zs,whenToUse:Vs,argumentHint:Ks,userInvocable:!0,getEffort(e,n){let{explicit:s}=Te(e);if(s===void 0)return;return tt(Ae(n?.options?mc(n):void 0),s)},getDefaultEffort(e,n){let s=Te(e),r=nt(s,n);if(r===void 0)return;let h=n?wo(s,n):r,y=Ae(n?.options?mc(n):void 0),w=fo(s,de[y][h].cell,n!==void 0&&ot(n)!==void 0,n),b=yo({...w,ignored:s.maxFindingsIgnored},"you");return{value:tt(y,r),notice:`${s.unrecognizedLevel!==void 0?`Ignoring unrecognized effort "${s.unrecognizedLevel}"; valid: ${he.join(", ")}. `:""}Reusing ${r} effort, the level you typed last time${h!==r?`; running at ${h} here`:""}. Type a level like \`/code-review high\` to change it.${b?` ${b}${w.reused?" Pass `--max-findings default` to go back to the usual limit.":""}`:""}`}},onUserTypedArgs(e,n){let{explicit:s,maxFindings:r}=Te(e);if(s!==void 0)Ys(s,n.storageV5);if(r!==void 0){let h=r==="default"?void 0:r;Pe((y)=>y.codeReviewLastMaxFindings===h?y:{...y,codeReviewLastMaxFindings:h},n.storageV5)}},getContext(e,n){if(Ua())return"inline";if(ot(n)!==void 0)return"inline";return"fork"},formatForegroundForkResult:Ft,getPromptForCommand:Xs})}