---
name: impeccable
description: Guidance-only Impeccable subset for UI critique, manual accessibility/responsive audit, and scoped polish, layout, typography or UX-copy refinement. Use for existing interfaces and Filament products while preserving approved FigCraft frames and DESIGN.md. Not for backend code review or executable Impeccable features.
license: Apache-2.0
---

# Impeccable — guidance-only adaptation

Locally modified from Paul Bakaus's Impeccable 4.8.0. See [UPSTREAM.md](UPSTREAM.md) for the immutable source revision, modified-file mapping, exclusions and update process. This is a small prompt/reference subset, **not the full Impeccable framework**. No executable engine, deterministic detector, hooks, telemetry, catalog access or live browser machinery is included. Do not claim any of those ran.

## Establish authority and scope

1. Read applicable global/project instructions, the current task, existing PRODUCT.md and DESIGN.md, and the target's components/tokens and neighboring flows. Missing documents are not permission to invent product truth or write replacements.
2. Approved FigCraft frames and the existing DESIGN.md are authoritative. Follow design-first/figcraft-design when applicable. If code, frames and written guidance disagree, report the conflict and obtain a decision instead of silently choosing a new design. No design for a requested UI change still means following the existing design-first gate.
3. Preserve the established identity, factual copy and behavior outside the explicit scope. Generic taste advice is a diagnostic lens, not a ban that overrides the approved system. For Filament products, prefer the product's Filament-native forms, tables, actions, navigation, overlays, components and tokens; do not replace them with a bespoke frontend or impose Storybook/new dependencies without the applicable separate approval.
4. Choose the surface's mode: **Operate** for task-oriented product UI, **Read** for comprehension, **Persuade** for an offer/action, **Experience** for the work itself. A product dashboard is not a marketing page. Read [reference/operate.md](reference/operate.md) for product and reading surfaces; marketing or showcase redesign is outside this subset.

## Supported actions

Use `/impeccable <action> <target>` for an explicit action. Read only the relevant reference. Normal skill discovery also works: when this skill is loaded for an ordinary UI task, use that task's existing scope and authority; do not pretend a slash command ran or turn loading guidance into permission to edit.

| Action | Contract | Reference |
| --- | --- | --- |
| `critique` | Read-only, evidence-based UX/design findings | [reference/review.md](reference/review.md) |
| `audit` | Read-only manual a11y, responsive, theming and performance review; no detector | [reference/review.md](reference/review.md) |
| `polish` | Small, explicitly requested refinement inside the approved system | [reference/polish.md](reference/polish.md) |
| `layout` | Scoped grouping, hierarchy, density and responsive structure | [reference/layout.md](reference/layout.md) |
| `typeset` | Scoped use of existing type roles and assets | [reference/typeset.md](reference/typeset.md) |
| `clarify` | Scoped UX-copy improvements preserving product meaning | [reference/clarify.md](reference/clarify.md) |

An explicit command with no action shows this table and the limitations, then stops. An unknown action, including upstream `init`, `document`, `shape`, `live`, `generate`, `detect`, `hooks`, `doctor`, `pin`, `unpin`, or any other unlisted action, explains that it is unavailable here and stops. Do not emulate an executable workflow, create aliases, regenerate DESIGN.md, run an installer, download assets or seek wider network permissions as a fallback.

## Work and verification boundaries

- Critique/audit inspect and report only: no fixes, document writes, build/test artifacts, or mutating checks. Separate verified findings from hypotheses and checks not run. A source inspection is not a rendered-browser test or detector scan.
- Refinement applies only the explicitly requested task's files/hunks after its design gates. Record the starting diff and preserve staged, unstaged, untracked and other-session work. Stop on ownership ambiguity or concurrent edits. Report potential behavior changes separately; do not smuggle them into visual cleanup.
- Keep current agent/model and existing tool, host, installation, spending, Git and security approvals. Do not dispatch workers, install/run an Impeccable engine, add hooks/plugins, alter permissions/providers/dependencies, fetch remote catalogs/fonts, post, commit, push, merge or deploy as a side effect. This skill grants no additional permission, including for otherwise useful checks.
- Use only already authorized inspection, browser and test tools. For edits, inspect check commands before running them and run relevant permitted verification; report missing tools or approvals rather than bypassing them. Never disable a test or invent a successful browser, accessibility or performance result.
- Report the exact target, evidence, findings or changed files, behavior/design/user work preserved, and actual verification/limits. Distinguish manual advice from automated results. This design guidance neither replaces /code-review or /simplify nor changes their parent-owned completion pipeline.
