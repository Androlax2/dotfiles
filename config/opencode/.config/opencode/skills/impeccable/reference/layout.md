# Layout and hierarchy

Modified guidance-only adaptation of upstream `skill/reference/layout.md`; detector commands, worker dispatch, live-mode parameters and automatic handoffs are removed. See [provenance](../UPSTREAM.md).

Layout turns task priority into reading order, grouping, rhythm and usable space. Preserve the established visual world and product-native controls.

## Assess with evidence

- **Reading order:** can the primary, secondary and major groups be identified at a glance?
- **Grouping:** do related items sit together, or are extra containers compensating for weak proximity?
- **Rhythm:** do tight and generous intervals communicate relationships instead of equalizing every element?
- **Structure and density:** do repeated cards/columns represent genuinely equivalent content? Does density suit task frequency and complexity? Filament tables, sections and navigation are not defects merely because they are familiar.
- **Adaptation:** what reflows, wraps, collapses or scrolls at supported sizes and zoom? Do visual and keyboard/DOM order agree?
- **Extremes:** test long content, localization, empty states, sticky/overlay elements and usable targets where authorized.

## Make a scoped correction

State the primary path, grouping relationships and intended density before changing structure. Use existing spacing/layout tokens and the simplest native primitive that expresses those relationships. Prefer proximity to unnecessary decoration, and component/container-aware reflow to arbitrary screen-specific positioning.

Variation is not a goal. Repetition supports recognition when the tasks are equivalent. Do not replace approved topology, navigation or a shared component without a design decision and the applicable authorization.

Verify the intended reading/task path at supported sizes with real content. Record rendered versus source-only evidence and remaining gaps. No detector scan is included or implied, and no subsequent action runs automatically.
