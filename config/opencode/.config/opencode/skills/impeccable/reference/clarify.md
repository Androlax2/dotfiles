# Clear interface language

Modified guidance-only adaptation of upstream `skill/reference/clarify.md`; automatic handoff is removed and task/design authority is explicit. See [provenance](../UPSTREAM.md).

Preserve factual meaning, domain terms, brand voice and approved copy. Read the whole interaction path rather than optimizing isolated strings. Ask before changing claims, legal meaning, translation contracts or domain-specific terminology.

## Audit the language

Look for ambiguous actions, unexplained jargon, vague states, missing consequences/recovery, redundant explanation and inconsistent terms. Infer audience and task from the actual product context; do not invent it.

For each state identify the fact needed now, the next available action, context that changes the decision and the appropriate tone. Say each idea once. A heading and its helper text should not repeat each other.

## Rewrite by function

- **Actions/navigation:** name the outcome with consistent verbs and objects. Destructive actions must communicate their real consequence; changing labels does not authorize changing confirmation or undo behavior.
- **Forms:** keep persistent labels and honest examples. Explain format/eligibility requirements before submission, with errors near the field and accessible announcements. Preserve validation and translation keys/contracts.
- **Errors/permissions:** explain what failed, why when known, and an actual recovery path. Do not promise an unknown cause or fix or expose internal codes as the primary explanation.
- **Loading/empty/success:** distinguish first use, no results, filters, permission limits and failure. Name the real operation; never invent progress or a successful outcome.
- **Help:** answer the user's likely question rather than restating the control. Keep links meaningful out of context and accessible names aligned with visible labels.

Use complete translatable messages and allow realistic expansion. Do not rely only on color, punctuation or iconography to carry meaning. Keep alt text informative where the image carries information and empty for decoration.

Verify comprehension, factual accuracy, terminology, recovery, realistic widths/zoom, localization and accessible names using permitted evidence. Report what was checked and what was not. Stop after the requested copy work; do not start a redesign or another command automatically.
