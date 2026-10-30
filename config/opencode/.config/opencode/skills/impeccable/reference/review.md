# Manual critique and audit

Modified guidance-only adaptation of upstream `skill/reference/critique.md` and `audit.md`; engine/scoring/storage/browser-launch workflows are omitted. See [provenance](../UPSTREAM.md). This is report-only, not a deterministic detector.

## Establish evidence

Name the target, task and relevant state/viewport. Read the approved visual authority and implementation first. Use an already authorized rendered view when available; otherwise state that conclusions are source-only. Do not launch a server, install a browser, write screenshots or claim interaction testing without its normal authorization. Do not alter the product or DESIGN.md during review.

## Critique: does the interface help its visitor?

- Identify the visitor's primary job and whether the first view makes it obvious. Check reading order, primary/secondary hierarchy, task paths and progressive disclosure.
- Evaluate grouping, density, spacing rhythm, typography and color against the actual product system, not generic marketing tastes. Familiarity is useful in Operate mode.
- Trace real content, empty/loading/error/success states and recovery paths. Check whether language and affordances communicate consequences accurately.
- Separate implementation drift from an unresolved design decision. Propose design changes rather than silently overruling an approved frame or native component convention.

## Audit: what can the evidence actually establish?

- **Accessibility:** semantics, names/labels, keyboard order, visible focus, contrast evidence, target sizes, status/error announcements, reduced motion and zoom. Do not assert measured contrast or screen-reader success without checking it.
- **Responsive behavior:** narrow/intermediate/wide containers, long or localized text, reflow, overflow, sticky/overlay behavior, input modes and visual versus DOM order.
- **Theming and states:** semantic tokens, light/dark variants actually supported by the product, selected/disabled/error states, and consistent component vocabulary.
- **Performance:** inspect asset sizing, layout shifts, unnecessary rendering and expensive effects. Distinguish likely costs from measured bottlenecks; no invented scores or synthetic detector results.

## Findings first

Rank actionable findings by effect: blocking task/accessibility loss, major friction, moderate drift, then polish. Include a precise file/selector/component or observed screen/state, the evidence, user consequence and smallest compatible recommendation. Mark uncertainty and coverage gaps. Avoid arbitrary scoring, speculative anti-slop bans and duplicate findings. End with checks actually performed and checks not run; do not apply fixes or automatically start another command.
