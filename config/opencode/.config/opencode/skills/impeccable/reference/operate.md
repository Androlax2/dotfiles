# Product and reading surfaces

Modified guidance-only adaptation of upstream `skill/reference/operate.md` and the Modes section of `skill/SKILL.src.md`; command links and universal prescriptions were replaced with project-authority guidance. See [provenance](../UPSTREAM.md).

For app UI, admin, settings, tables and tools, design serves a task. A category-fluent user should recognize controls and trust the interface without learning a decorative visual language. On Read surfaces, comprehension and wayfinding come first.

- Keep the established component vocabulary. In Filament, use existing forms, actions, tables, resource navigation, overlays and theme tokens rather than rebuilding standard controls for flavor. Familiarity, density and consistent affordances are strengths here.
- Use existing type roles and readable measures. One well-tuned family may be right for product UI; a new display/body pairing is not automatically an improvement. Prose and data have different density needs.
- Color should carry stable meaning: primary actions, selection, status and data. Preserve semantic tokens and established contrast, rather than decorating inactive controls with strong accents.
- Make responsiveness structural: adapt navigation, columns, tables and disclosure to actual task priority. Preserve keyboard/DOM order and usable targets, not just a visually attractive screenshot.
- Verify applicable loading, empty, error, success, focus and disabled states. Follow the product's loading and recovery conventions instead of imposing one universal skeleton/spinner or modal rule.
- Motion should explain state and remain interruptible. Respect reduced motion and existing timing; do not add page-load choreography or exotic effects to a task UI.
- Reading interfaces need a clear location, heading structure, comfortable copy and a predictable way forward. Monospace belongs where code/data benefits, not as a costume for a technical product.

These are review lenses, not permission to alter approved frames or written decisions. If a system choice appears to harm usability, report it with evidence and ask the design owner to resolve it.
