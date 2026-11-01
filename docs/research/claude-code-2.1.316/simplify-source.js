var qi=`\`/simplify → 4 cleanup agents in parallel → apply the fixes\`

You are improving the quality of the changed code, not hunting for bugs. Review
it for reuse, simplification, efficiency, and altitude issues, then fix what you
find. Do not look for correctness bugs — that is what \`/code-review\` is for.

${Y}
## Phase 1 — Review (4 cleanup agents in parallel)

Launch **4 independent review agents** via the ${mt} tool, all in a
single message so they run concurrently. Pass each agent the diff and one of
the four angles below. Each returns its findings with \`file\`, \`line\`, a
one-line \`summary\`, and the concrete cost (what is duplicated, wasted, or
harder to maintain).

### Reuse

${me}
${Z}
${oe}
${ne}
## Phase 2 — Apply the fixes

Wait for all four agents to complete, dedup findings that point at the same
line or mechanism, and fix each remaining one directly. Skip any finding whose
fix would change intended behavior, require changes well outside the reviewed
diff, or that you judge to be a false positive — note the skip rather than
arguing with it. Finish with a brief summary of what was fixed and what was
skipped (or confirm the code was already clean).
`,zi=`\`/simplify → ${mt} tool unavailable → single-pass inline cleanup → apply the fixes\`

You are improving the quality of the changed code, not hunting for bugs. Review
it for reuse, simplification, efficiency, and altitude issues, then fix what you
find. Do not look for correctness bugs — that is what \`/code-review\` is for.

The ${mt} tool isn't available in this context, so the usual
4-agent fan-out can't run. Work through all four angles below yourself, in
this same context, in one pass — do not skip an angle for lack of fan-out.

${Y}
## Phase 1 — Review (4 cleanup angles, single pass)

Review the diff against each angle below in turn. For each, note findings with
\`file\`, \`line\`, a one-line \`summary\`, and the concrete cost (what is
duplicated, wasted, or harder to maintain).

### Reuse

${me}
${Z}
${oe}
${ne}
## Phase 2 — Apply the fixes

Dedup findings that point at the same line or mechanism, and fix each
remaining one directly. Skip any finding whose fix would change intended
behavior, require changes well outside the reviewed diff, or that you judge to
be a false positive — note the skip rather than arguing with it. Finish with a
brief summary of what was fixed and what was skipped (or confirm the code was
already clean). State clearly in your summary that this was a single-pass
review done without the ${mt} tool, not the full 4-agent
fan-out, so whoever reads it isn't misled about what actually ran.
`;function an(){ol({name:YSn,menuDescription:"Clean up the changed code without changing behavior",description:"Review the changed code for reuse, simplification, efficiency, and altitude cleanups, then apply the fixes. Quality only — it does not hunt for bugs; use /code-review for that.",argumentHint:"[<target>]",userInvocable:!0,async getPromptForCommand(e,n){let s=e.trim(),r=s?`Review target: \`${s}\`

`:"",h="",y=Ee(n)?qi:zi;return[{type:"text",text:`${r}${h}${y}`}]}})}