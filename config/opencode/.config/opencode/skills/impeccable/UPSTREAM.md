# Upstream and local modifications

This is **Impeccable-derived, guidance-only material**, not an installation of the executable Impeccable framework.

- Project: Paul Bakaus, [pbakaus/impeccable](https://github.com/pbakaus/impeccable).
- Reviewed revision: [`df3a4abe7d0840d5f60324812c432e00b66a4e75`](https://github.com/pbakaus/impeccable/tree/df3a4abe7d0840d5f60324812c432e00b66a4e75), 2026-10-30, skill version **4.8.0**.
- Canonical content: `skill/SKILL.src.md` and `skill/reference/`. `.opencode/skills/impeccable/` and `.opencode/commands/impeccable.md` were inspected as the upstream OpenCode distribution, not executed.
- License: upstream [LICENSE](LICENSE), Apache-2.0, including its copyright notice. Upstream [NOTICE.md](NOTICE.md) is retained unchanged. Native iOS/Android references mentioned there are not included in this subset.

## Modified-file mapping

All instructional files here are local adaptations, not verbatim upstream releases. The single V2 command is a local adapter for the upstream namespaced `/impeccable <action> <target>` entry.

| Local file | Upstream input at the pinned revision | Changes |
| --- | --- | --- |
| `SKILL.md` | `skill/SKILL.src.md` and `.opencode/skills/impeccable/SKILL.md` | Narrow description/routing to six manual actions; remove executable grants/setup and all runtime orchestration; add FigCraft/DESIGN.md/Filament and authorization boundaries |
| `reference/review.md` | `skill/reference/critique.md`, `audit.md` | Condense assessment themes into report-only manual reviews; omit engine, deterministic/scored review, storage, dashboards and automated browser workflows |
| `reference/operate.md` | `skill/reference/operate.md`, Modes in `skill/SKILL.src.md` | Condense product/read guidance; make conventions subordinate to actual product/design authority, not universal bans |
| `reference/polish.md` | `skill/reference/polish.md` | Remove context/detector/storage calls and automatic handoffs; constrain edits and verification |
| `reference/layout.md` | `skill/reference/layout.md` | Remove detector commands, agents, variant parameters and handoffs; preserve approved topology |
| `reference/typeset.md` | `skill/reference/typeset.md` | Remove detector/agent/live/identity-replacement workflows; preserve approved type and existing assets |
| `reference/clarify.md` | `skill/reference/clarify.md` | Condense copy guidance; preserve domain meaning, translation contracts and task scope; remove handoff |

## Included and unavailable

Included: one OpenCode V2 command, one normally discoverable skill, six Markdown references and the license/notices. The payload has no scripts, binaries, downloaded fonts/assets, browser injection, hooks, plugin registration or executable tool grants. No dependency or global provider/permission configuration is needed.

Unavailable: engine/context automation, deterministic slop detection, stored/scored critique, hooks, telemetry, catalog requests, setup/update/doctor/pinning, live/generate, image production, design-world generation, and the other upstream actions. The namespace remains recognizable, but there is no claim of full upstream command parity or measured design improvement. An unavailable action stops with an explanation; it must not install or run upstream tooling as a fallback.

The upstream launcher can download an engine binary on first use. Current upstream workflows can fetch visual catalogs and emit telemetry; hooks can run outside model approval. Those capabilities need a separate bounded proposal covering exact binaries/checksums, hosts, writes, hook/approval behavior and rollback before any future installation. This integration does not change security settings to disable them: it simply does not ship them.

## Overlap and updates

This adds design guidance alongside frontend-design, baseline-ui, improve-ui, fixing-accessibility and create-design-md; it replaces none of their IDs. It does not replace design-first or FigCraft approval, create a new authoritative design document, redesign a product, or compete with Filament's native conventions. The generic `/code-review`, `/simplify` and global completion-rule task `38a4e84f` are unchanged. No top-level action aliases are created.

Update manually: inspect a new immutable upstream revision and its license/notices, review the mapped source changes, update only this approved static subset and this provenance record, then rerun regression/native discovery checks and reviewed cockpit application. Do not run the upstream installer or updater. Do not add executable material by copying a newer skill distribution over this directory.

Discovery tests establish configuration/payload integrity and native registration, not model compliance or superior design output. No paid model benchmark or new worker is part of installation. The user's project and design owners remain responsible for evaluating any future design recommendations.
