# One status and one next step for disk maintenance

Written against: `77b42abb9d14df477899976dcf32983ac0f162fb`.

## Design language

- Audited surface: Vitals' French Disques summary/review/details family.
- Design sources: the user's screenshot and pasted diagnostic output; the user explicitly accepted a guided default with one plain-language status, one recommended action, and all technical diagnostics/controls under Details.
- Documented decisions: preserve the seven tabs, existing native severity colors, real SMART/space warnings, and non-destructive report services.
- Governing owners and consumers: `apps/vitals/src/tui.rs` (`draw` → `disks` → summary/review/details; `header`; `handle_key_with`). The data owner is the unchanged `maintenance.rs` snapshot.
- Explicit exceptions: critical machine-wide alerts must remain visible on every tab.

## Findings

| # | Problem | Evidence | Proposed change | Scope | Confidence |
| --- | --- | --- | --- | --- | --- |
| 1 | The default flow still makes the user choose between tools and interpret diagnostics. | User says the interface is hard to understand and selected one-action guidance. `disk_actions` creates three choices; `disk_summary` renders all of them and a schedule; `disk_review` renders counts and diagnostic paths ahead of the useful volume summary. These reach the Disques family directly through `draw`/`disks`. | Use one primary recommendation in Summary, a plain-language Review, and move diagnostics/manual tool choices to Details. | Disques family and its summary header/footer only. | High |

## Improve first

Change the entire default flow, not just its labels: one next step must not open another wall of technical diagnostics.

## Evidence chain

- Surface: Disques summary, review, technical details, header, and keyboard routing.
- Problem: paths, error totals and three similarly weighted checks obscure what the user should do. The provided paste is the technical-details presentation; moving only the same data between pages is not enough.
- Design evidence: latest explicit user approval of a guided mode; existing `DiskOverview.primary` already identifies one recommendation and `DiskScreen` already supports optional details.
- Owner: `apps/vitals/src/tui.rs`.
- Scope: that file, its existing rendered/key tests, and the relevant README instructions.
- Uncertainty: an inspected open Vitals process used a replaced executable inode. That does not prove which layout it displays; validate a fresh window after separately approved deployment, without moving or closing existing windows automatically.

## Design decision

Summary answers “Is anything urgent?” in one plain French sentence and offers at most one contextual primary action. This follows the accepted “one recommended action—or nothing needed” scope: a healthy-state review is explicitly optional, and an already-running first scan requires no further action. No raw paths, error totals, scan counts, timestamps, schedules, service names, or advanced checks appear in Summary or Review. Percent used and human-readable volume sizes are useful product information, not diagnostics.

Keep existing priority and severity rules; never turn unknown data or actual failures into fabricated health/success. For a partial scan, use count-free copy such as “Le bilan est incomplet : certains dossiers n’ont pas pu être mesurés. Aucun fichier n’a été modifié.” Do not claim that every error is harmless or that protected folders must have their permissions changed.

## Reuse

- Reuse `DiskOverview.primary`, `DiskScreen`, `DiskAction`, `wrap_disk_lines`, `level_style`, the native snapshot, and the two typed `Check` variants.
- Keep `maintenance::request_check`, running-job guards, request feedback and asynchronous semantics. No service/repository abstraction or new dependency.
- Exemplar: current paragraph/block rendering, reversed primary-action style, and Details scroll behavior in `tui.rs`.

## Changes

1. `apps/vitals/src/tui.rs`: Summary.
   - Render a short status plus at most two short explanatory lines and one highlighted contextual recommendation.
   - Use existing primary action; ignore stale selection of a now-hidden secondary action. Home/End/arrows must never expose or invoke hidden controls.
   - Healthy state uses an explicitly optional heading (for example “Si tu veux libérer de la place”), not an apparent to-do. When the first space review is already running, show waiting/reassurance text and no redundant “Préparer le bilan” recommendation or Enter instruction. A previous review may remain an optional read action while a newer one runs. Preserve actual warning precedence.
   - A contextual first-report action may request only the existing read-only space review, with plain “Préparer le bilan” wording. Turbo and the general manual-check chooser are advanced controls only.
   - Remove schedule/error/diagnostic counts from this flow. Keep partial/unknown caveats as short language, not data dumps. Preserve critical warnings over optional request feedback.
2. `apps/vitals/src/tui.rs`: Review.
   - Show the largest known games/projects with short, unambiguous names and sizes; keep parallel checkout project context and partial-size labels.
   - Replace diagnostic lists/counts/timestamps with one short incomplete-data note when needed and a Details affordance. Preserve “large does not mean unused” and no automatic removal.
3. `apps/vitals/src/tui.rs`: Details and key routing.
   - Retain all native results, counters, paths, schedules and raw diagnostics here.
   - Keep both read-only reports reachable using clearly displayed Details-only controls (for example `a`: space review, `t`: Turbo report). Reuse safe dispatch, never arbitrary unit strings.
   - Preserve running-check protection, queue-versus-completion distinction, error feedback, and refusal to add scanning load when the existing SSD-critical guard disallows checks.
   - Keep `d` for Details, `b`/Backspace back, tab keys and Care behavior. Opening/refreshing still starts nothing.
4. `header`/footer in `tui.rs`.
   - In both guided Summary and Review, replace technical alert totals with a short machine-wide state. Critical alerts stay prominent/red (e.g. “PC : alerte importante”); warnings/unknowns remain explicit, never falsely “tout va bien”. Other tabs and Details preserve their current counters.
   - Summary footer: Enter for the one recommendation, d for Details, r for state refresh, q quit; narrow terminals must retain essentials.
5. Tests and `README.md`.
   - Update old three-choice tests to the newly approved one-choice contract, not disable them. Keep advanced action safety coverage.
   - Explain the simple default and optional Details/manual checks without instructing users to change permissions or delete projects.

## Scope

- Inherit: Disques summary/review/details only.
- Verify: shared header in other tabs, Care keys, narrow terminals, real warning precedence, native command routing.
- Exclude: producer/schema/retention changes, root units, automatic scans, permission fixes, dependency manifests, unrelated host work, `.handoff/`, git writes and deployment.

## Validation

- Product: normal state gives one clear next step; report limitations do not masquerade as a hardware diagnosis; critical SSD/95% space warnings remain immediate.
- Interface: render healthy, 85%/95%, critical/stale/missing SMART, partial/legacy/missing/invalid reports, real read issues, running checks and rejected requests at 100x26 and 40x12.
- Interaction: assert exactly one primary recommendation; Summary/Review contain no diagnostic paths/counts/schedules; stale hidden selections cannot launch; Details controls target only typed user reports, cannot repeat running jobs or bypass the SSD guard; return/tab/Care keys work.
- System: reuse native model and styles; no new widget system or helper daemon.
- Repository: authorized `cargo test --offline --manifest-path apps/Cargo.toml -p vitals`, full `CARGO_NET_OFFLINE=true make test`, Python fixtures, and `git diff --check` → pass, or record actual failures without suppression.
- Use TDD: failing rendered/key regression first, observe RED, implement the smallest correction, then verify GREEN. Host builds/tests must use `ask-to-run` approval; no install or real scan is included.

## Stop conditions

- Stop if implementing this requires changing the producer, hiding genuine failures, weakening safety guards, adding dependencies, or changing installed settings.
- Preserve unrelated changes and untracked `.handoff/`. Do not commit, fetch/rebase, apply patches or restart apps without separate permission.

## Design documentation

After acceptance and validation, record the guided one-action decision in the existing README section. Keep earlier plans as history; no separate design-system document is needed.
