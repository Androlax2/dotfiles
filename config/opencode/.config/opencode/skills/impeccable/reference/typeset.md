# Typography within the product's identity

Modified guidance-only adaptation of upstream `skill/reference/typeset.md`; detector commands, identity-replacement routing, worker dispatch, live-mode parameters and automatic handoffs are removed. See [provenance](../UPSTREAM.md).

Improve the use of established fonts and roles, not the identity itself. Read approved typography, DESIGN.md, tokens and existing font assets first. Do not download or substitute fonts as a fallback.

## Assess

- Identify heading, body, label, metadata, code and data roles and their actual usage.
- Check whether size, weight, spacing and tone make role hierarchy legible without unnecessary families or weights.
- Inspect repeated-role consistency, reading measure, line height, contrast, tracking and paragraph rhythm in real content.
- Stress long headings, narrow containers, localization expansion, zoom, missing weights and fallback metrics.
- Inspect existing loading/fallback behavior for invisible text or disruptive reflow. A source suspicion is not a measured performance result.

## Refine

Use the fewest existing roles that express the hierarchy. Preserve the product's density and platform conventions: comfortable prose and compact data need not share one measure. Keep body text zoomable, let headings wrap coherently, and test deliberate line breaks across supported widths. Respect browser zoom and user font settings.

Use tabular/numeric/code features where the content benefits. Apply modern wrapping properties only when compatible with the existing stack and browser targets; do not introduce hard breaks, font fetching, framework swaps or token changes that contradict the approved design.

## Verify

Check hierarchy, long text, contrast, font fallback/loading and user text scaling with already authorized tools. Record which states and widths were actually checked and what remains unverified. No automated typography detector or live variant generator is part of this adaptation.
