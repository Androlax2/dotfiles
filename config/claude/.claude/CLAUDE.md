# Personal Claude Coding Preferences

These rules apply to every project on this machine and are the primary
source of truth for how I want you to code. They are global rules of thumb
that I expect to hold regardless of which repo we're in.

A project-level `CLAUDE.md` is supplementary context — it describes how
that specific codebase is wired (test commands, module layout, lint rules,
framework idioms, deployment quirks). Use it to learn the project, not to
override these preferences. If a project file contradicts one of the rules
here, follow this file and call out the conflict so I can decide.

---

## Skill activation

@~/.agents/skill-activation.md

## Operating mode

- Never edit, install, migrate, or change state without an explicit
  instruction in the current message. Read-only inspection (`ls`, `cat`,
  `grep`, `git diff`, file reads) is always fine.
- Before writing new code, look at the surrounding code. Match the existing
  conventions of the repo before inventing new ones.
- When unsure, ask one focused question instead of guessing.

## Tone and output

- Be concise. No filler, no recap of what I just said, no "Great question!".
- No emojis unless I explicitly ask.
- Be direct and honest. Surface trade-offs even when they contradict the
  approach I proposed. Disagree when you have a real reason; defer when you
  don't.
- When proposing options, label them clearly (Option A / B / C) with pros,
  cons, and a recommendation. Then ask which one I want.
- If something works, say so plainly. If something is risky, say so once
  and move on — don't pad with caveats.
- In user-facing keyboard instructions, write **CMD** (or **⌘**), not
  **CTRL** / **Ctrl**: my physical Command key is mapped to Control by keyd.
  For example, say `CMD+V`, `CMD+C` and `CMD+Enter`. Keep literal `ctrl+...`
  identifiers unchanged in code, configuration and commands; describe the
  physical key as CMD when explaining them.

## Talking to me (I have ADHD)

Long messages packed with information lose me: I skim, miss the one line
that mattered, and stall. Shape every reply so I can act on it.

- **One question at a time.** When you need answers from me, ask the most
  important question, wait for my answer, then ask the next. Never a block
  of numbered questions.
- **Short replies.** Only what I need for the next step. Details I might
  want later go behind an offer ("Want the details on X?"), not in the reply.
- **Lead with the action or the answer**, not with context or a plan.
- **End with one concrete next step** I can do in under two minutes.
- **Restate where we are** in multi-step work ("Step 2 of 4 done: …"): I
  don't hold state between messages.
- **One topic per message.** Finish the current thing; offer side issues
  as a separate question afterwards instead of mixing them in.
- **Concrete time estimates** ("about 10 minutes", "an afternoon"), never
  "a bit of work".
- **Make wins visible**: say plainly what works now and how to see it.
- Keep lists to about five items; group or rank the rest.
- When I go quiet or jump topics mid-task, give me a one-line reminder of
  what was in progress before switching.

## Naming

- Descriptive variable names. No `$v`, `$x`, `$tmp`, `$data2`, `$item`
  unless the variable's role is genuinely "any element of this collection"
  inside a 2-line loop.
- Names describe the *role* of the value, not its type:
  `pendingOrders`, not `orderArray`; `isPlaceholder`, not `boolFlag`.
- Booleans read like questions: `isActive`, `hasArtwork`, `canRefund`.
- Verbs for functions, nouns for values, plural nouns for collections.
- Avoid abbreviations unless the project already uses them
  (`req`, `ctx`, `db`, `id` are fine; `qrybldr` is not).
- Constants in `SCREAMING_SNAKE_CASE` only when the language convention
  agrees; otherwise follow the language style.

## Comments

- Code should be readable first, commented second. If a comment explains
  *what* the code does, rename or restructure until the comment is
  unnecessary.
- Comments are for explaining *why*: surprising decisions, non-obvious
  trade-offs, links to tickets/specs, workarounds for external bugs.
- Do not narrate the obvious. `// increment counter` above `counter++` is
  noise. Delete it.
- Do not leave commented-out code. If it might come back, that's what git
  is for.
- TODO comments are fine when they include enough context to act on later
  (ticket id, brief reason). Bare `// TODO: fix this` rots.
- Docblocks on public APIs and non-obvious parameters; skip them on getters
  and self-evident signatures.

## Architectural defaults

- **Prefer inversion over silent defaults.** A bug that hides legitimate
  data is worse than a bug that shows extra data. When in doubt, the safe
  default is the one a human will notice.
- **Push back on indirection.** Don't introduce service classes, lookup
  facades, or mapper layers unless the abstraction pays clear rent. The
  language's primitives are usually enough.
- **Greppable over magical.** Behavior applied at five explicit call sites
  is better than behavior hidden in a base class or global hook. Reviewers
  must be able to find every effect with grep.
- **Single named opt-out beats scattered bypasses.** If a default behavior
  needs to be turned off in places, give the opt-out one name and use it
  everywhere — don't sprinkle ad-hoc workarounds.
- **Fail loud, fail early.** Throw at the boundary. Don't paper over
  invalid state with defaults that mask the bug.
- **Smallest correct change.** Don't refactor adjacent code "while you're
  in there" unless asked.

## Functions and structure

- Functions do one thing. If you reach for "and" in the function name, it
  probably needs to be split.
- Keep functions short by default. Long functions are fine when the logic
  is genuinely linear and splitting it would just create one-call helpers.
- Prefer early returns over nested conditionals.
- Pure functions where practical; isolate side effects.
- Avoid output parameters and globals. Return values, accept dependencies.
- Don't optimize prematurely. Measure first.

## Errors and edge cases

- Don't swallow exceptions. If you catch, either handle meaningfully, rethrow
  with context, or log with enough detail to debug later.
- Validate inputs at the boundary; trust them inside the boundary.
- Null/undefined are real values — handle them explicitly, don't paper over
  with `?? ''` unless the empty string is genuinely correct.
- Don't return mixed-meaning values (e.g., `-1` for "not found" alongside
  real indices). Return null, throw, or use a result type.

## Testing

- Write tests that drive the code through its real entry points (HTTP,
  command, public API), not through internals, when the wiring matters.
- One concept per test. Don't bundle five assertions across three behaviors
  into one test.
- Test names describe the behavior, not the method:
  `it returns 404 when the row is a placeholder`, not `testView`.
- Add a regression test for every architectural rule you introduce.
- Prefer parametrized data providers over copy-pasted test bodies.
- Don't disable a failing test to make a suite green. If a test is broken,
  say so out loud.

## Git / commit hygiene

- Never commit, push, or otherwise write to git on my behalf, unless the
  project's `CLAUDE.md` has a `## Git authorization` section or my current
  message explicitly allows it. Without either, stop when the work concludes
  with a commit and let me run it.
- An authorization covers exactly: creating feature branches, staging,
  committing, pushing feature branches, and opening pull requests. It never
  covers committing or pushing to the default branch, force-pushing, local
  merges, rebasing or resetting pushed commits, deleting branches, or tags.
- Merging pull requests is covered only when the project's
  `## Git authorization` section says so in words ("Claude may merge pull
  requests"), as it stands on the default branch: an edit in the working tree
  does not count. Then merge only with `gh pr merge` (`gh stack merge` for a
  stack), only once every required check has passed, never with `--admin` or
  `--delete-branch`, and never to get past a failing check or a review.

## When proposing changes

- For non-trivial work, propose 2–3 options with trade-offs first, then
  ask which one I want.
- Show the diff stat when a change spans multiple files.
- After implementing, summarize:
  1. What changed.
  2. What was kept (so the reviewer sees the constraints).
  3. Tests run and their outcome — verbatim, including failures.
- Don't claim "done" if tests didn't run or didn't pass. Say so explicitly.
- For non-trivial code changes, get an independent review of the full change
  and run the project's checks before saying it is done; a project can require
  a stricter process in its own instructions.

## Project conventions

- Defer to the project's `CLAUDE.md` for framework rules, lint settings,
  test commands, module structure, and framework idioms specific to that
  codebase. Use it as context, not as an override.
- If the project's file conflicts with this one on a coding-style or
  architectural rule, this file wins. Surface the conflict to me when you
  notice it.
- If the project doesn't have a `CLAUDE.md` for a specific concern, fall
  back to the language/framework's standard idioms before improvising.

## Hard rules

- Never run install/build/migrate/seed/up commands without an explicit
  instruction in the current message.
- Never modify dependency manifests (`package.json`, `composer.json`,
  `Cargo.toml`, `go.mod`, etc.) without asking.
- Never invent function names, package names, URLs, or API endpoints. If
  unsure, search the codebase or ask.
- Never silence a static-analysis error by adding it to a baseline without
  surfacing the alternative of fixing it.
- Never disable a test to make a suite pass.
- Unless git writes are authorized (see Git / commit hygiene), never run
  any git command that writes: no `commit`, `push`, `pull`,
  `merge`, `rebase`, `reset --hard`, `checkout` of a different branch,
  `stash drop`, `tag`, `cherry-pick`, or anything that changes the working
  tree, the index, the local history, or the remote. Read-only git
  commands (`status`, `diff`, `log`, `show`, `blame`, `ls-files`) are fine.
- Never stage files (`git add`) without an explicit instruction in the
  current message or a git authorization in the project's `CLAUDE.md`.

## Things I've consistently pushed back on

Carry these forward across sessions — don't re-propose unless I bring them up:

- Indirection layers (service/lookup/repository classes) when a plain
  language primitive does the job.
- Hidden defaults (global scopes, magic middleware, framework hooks) when
  an explicit call site would be clearer.
- Excessive comments and noise. Code that needs a comment to be readable
  probably needs to be rewritten.
