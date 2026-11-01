# How Claude Code 2.1.316 runs `/code-review` and `/simplify`

Research for todo a198fb92: the behaviour to reproduce 1:1 in OpenCode, Codex and a
generic AGENTS.md section.

## Sources

Both commands are built into the CLI, not installed skills: no file under
`~/.claude/skills` or `~/.claude/plugins` defines them. Their prompts are template
strings in the binary (`~/.local/share/claude/versions/2.1.316`, around byte
293,050,000 for `/code-review` and 293,188,000 for `/simplify`).

- `code-review-source.js`: the extracted `/code-review` module (shared blocks, every
  level's recipe, model routing, argument parsing, flags, registration).
- `simplify-source.js`: the extracted `/simplify` prompt and registration.
- `code-review-observed-medium-prompt.md`: the exact prompt a forked `/code-review medium`
  received in this session (claude-opus-5-5), read from the subagent transcript.

Not yet done: the planted-diff observation runs (todo step 2) and the Anthropic docs and
changelog (step 1); only the binary and live transcripts back this spec.

## `/code-review`

**Arguments**: `[low|medium|high|xhigh|max|ultra] [--fix] [--comment] [<pr#>|<branch>|<path>] [--max-findings <n>|all|default]`.
Alias `/review`. The level must be the first token; a word that looks like a level but
isn't one (`med`, `hig`) is reported as ignored. With no level, the level the user typed
last time is reused (stored as `codeReviewLastEffort`), otherwise the session effort,
otherwise `medium`. `--max-findings` is remembered the same way; `default` clears it.
Only a level the *user* typed is remembered (`onUserTypedArgs`).

**Where it runs**: a forked subagent (`getContext` → `fork`) that returns findings to the
parent, unless the host renders the `ReportFindings` tool (desktop/web, or
`CLAUDE_CODE_REPORT_FINDINGS`), then inline. The model's effort is set to the level.

**Level → recipe depends on the model family.** The same level runs a different
recipe per model:

| Level | default family (this session) | claude-opus-5 | claude-sonnet-5 |
| --- | --- | --- | --- |
| low | `low`: 1 diff pass | `low` | `low-sonnet5` |
| medium | `o48-high-v1` | `o5-bmin` | `medium` |
| high | `o48-high-v1` | `o5-bmin` | `high` |
| xhigh | `o48-xhigh-v1` | `o48-xhigh-v1` | `xhigh` |
| max | `max` | `max` | `max` |

So in this session `medium` and `high` are the **same** prompt, and its header reads
"high effort → 8 inline angles → dedup (no verify)". That is why every medium run here
looked like a high one.

**The recipe families**:

1. *Low* (`low`, `o48-low-v1`, `low-sonnet5`): one tool call reads `git diff
   @{upstream}...HEAD; git diff HEAD`; one findings pass on hunks alone (wrong
   conditions, off-by-one, null deref, removed guard, falsy zero, missing await,
   copy-paste variable, swallowed error, duplicated helper, dead code). No subagents,
   no full-file reads, no style/perf/tests. Some variants skip test files. Cap 4 or 8.
2. *Inline angles, no verify* (`o48-med-v1`, `o48-high-v1`, `o48-xhigh-v1`): Phase 0
   gathers the diff; Phase 1 runs 8 angles (10 at xhigh) **in the same context, no
   subagents**, up to 6 candidates each (8 at xhigh); Phase 2 dedups only, no
   verification; xhigh adds Phase 3, a gap sweep. Cap 8 (medium), 10 (high), 15 (xhigh),
   with a floor of half the cap.
3. *Fan-out with verify* (`medium`, `high`, `xhigh`, `max`): the same angles, each run
   as its own subagent via the Agent tool; then one verifier subagent per candidate
   votes CONFIRMED / PLAUSIBLE / REFUTED (high and up are biased toward PLAUSIBLE);
   xhigh and max add a sweep finder. Sonnet 5 at high and up gets a hint of 2–8 finder subagents scaled
   to diff lines / 150. Without the Agent tool, it degrades to one inline pass and must
   say so in its summary.
4. *Minimal* (`o5-bmin`): one careful diff pass by a "senior engineer", no angles,
   results through `ReportFindings`, then restated as `file:line — summary` lines.

**The angles** (shared text, verbatim in `code-review-source.js`):

- Correctness: A line-by-line diff scan (reads the enclosing function too), B
  removed-behaviour auditor, C cross-file tracer (callers and callees); xhigh/max add
  D language pitfalls and E wrapper/proxy correctness.
- Cleanup: Reuse, Simplification, Efficiency.
- Altitude: root cause at the right depth, not special cases.
- Conventions: read `~/.claude/CLAUDE.md`, the repo-root CLAUDE.md and every
  CLAUDE.md/CLAUDE.local.md in an ancestor directory of a changed file; flag only a
  violation it can quote rule and line for.

Correctness outranks cleanup when the cap forces a cut. Finders must pass every
candidate with a nameable failure scenario through.

**Output**: a JSON array `[{file, line, summary, failure_scenario}]`, most severe first,
capped; the host pretty-prints it as a numbered list. With `ReportFindings`, the same
fields plus `short_summary` (≤60 characters) and a `category` slug, and later a second
call carrying each finding's `outcome` (`fixed`, `no_change_needed`, `skipped`).

**Flags**: `--fix` applies the findings after the report (skipping ones that change
intended behaviour, reach well outside the diff, or look false), then summarises fixed
and skipped. `--comment` posts inline comments on a GitHub PR (inline-comment tool,
else `gh api`) or one general note on a GitLab MR via `glab`; elsewhere it prints and
says the flag was ignored. `ultra` is a cloud review; when it can't run, it falls back
to a local review and says why. After a review of a diff with a runtime surface, it may
invoke a run-the-app skill if one exists and hasn't run.

**Never**: it changes no files without `--fix` and posts nothing without `--comment`.

## `/simplify`

**Arguments**: an optional target. No levels, no flags.

**Prompt**: "improving the quality of the changed code, not hunting for bugs". Phase 0 is
the same diff gathering as `/code-review`. Phase 1 launches **4 review subagents in
parallel, in one message**: Reuse, Simplification, Efficiency, Altitude (the same angle
text as `/code-review`), each returning `file`, `line`, `summary` and the concrete cost.
Phase 2 waits for all four, dedups, and **applies the fixes directly**, skipping ones
that change intended behaviour, reach well outside the diff, or look false; it ends
with what was fixed and skipped, or that the code was already clean.

Without the Agent tool: one inline pass over the four angles, and the summary must say
it was a single-pass review.

It runs inline (no fork), does not run tests, and does not look for correctness bugs.

## Ports

The OpenCode ports (`config/opencode/.config/opencode/commands/code-review.md` and
`simplify.md`) copy the default-family recipes above: low → Low (cap 4), medium and
high → 8 inline angles with no verify (cap 10), xhigh → 10 inline angles with a sweep
(cap 15), max → Claude's own single-pass fallback for max (10 angles, sweep, cap 15,
saying it ran without the fan-out). `/simplify` fans out to 4 subagents when the harness
offers a subagent tool, else runs Claude's single-pass fallback.

Known differences from Claude Code:

- No remembered level or `--max-findings`: with no level, the port uses medium.
- The model's reasoning effort is not raised with the level (the command frontmatter
  must not override models).
- Output stays raw JSON; Claude's host pretty-prints it as a numbered list.
- The Conventions angle also reads AGENTS.md files, which is where OpenCode looks.
- No `ultra`, no ReportFindings tool, no run-the-app follow-up.
- Codex (0.161 has no custom-prompt folder) gets the same text as skills in
  `config/codex/.codex/skills`, stowed to `~/.codex/skills`, invoked as `$code-review`
  and `$simplify`. A skill runs in the current session, not as a separate review
  subagent like Claude's fork. `agents/openai.yaml` turns off implicit invocation, so
  Codex never starts them unasked. `codex_skills.rs` fails when they drift from the
  OpenCode commands. Nothing yet checks that Codex itself lists them.
