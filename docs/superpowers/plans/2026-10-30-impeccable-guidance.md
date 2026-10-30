# Bounded Impeccable Guidance Plan

**Task:** `4f29b967`, existing owner `ses_e76fb955dffeoX2F5hiumfwSwO`; native execution only, no new workers.
**Goal:** Make useful Impeccable design guidance available as `/impeccable` and the normally discoverable `impeccable` skill without installing its executable framework.
**Authority:** Theo's integration request and “Continue”; routine implementation is authorized, host application remains separately reviewed.

## Research and boundary

Upstream inspected at [df3a4abe7d0840d5f60324812c432e00b66a4e75](https://github.com/pbakaus/impeccable/tree/df3a4abe7d0840d5f60324812c432e00b66a4e75), version 4.8.0 (2026-10-30). `skill/SKILL.src.md` and `skill/reference/` are the source material; `.opencode/` is its tool-specific distribution. The upstream OpenCode command uses deprecated `subtask`, selects `build`, and calls a skill using a different tool argument shape. Its setup runs an engine launcher, which can download a binary into the user's cache; playbooks call detector/storage commands, and broader workflows include hooks, remote catalogs and telemetry. None will be installed or invoked here.

Provide a prominently labeled **guidance-only local adaptation**, not full Impeccable. Retain upstream's namespaced `/impeccable <action> <target>` interface for six manual actions: critique, audit, polish, layout, typeset and clarify. Critique/audit are report-only; refinement needs a concrete authorized task and the existing design gates. No argument lists help and stops. Unsupported engine, setup, live, generation, pin/hook/doctor and other actions explain the limitation and stop—no install fallback.

Read project instructions, approved FigCraft frames and DESIGN.md first. Resolve disagreements rather than using generic advice to override them. Filament products retain their native components, navigation, actions, forms, tables, overlays and tokens. Missing designs still follow design-first; this skill grants no redesign, document replacement, security/dependency/provider changes, permission changes, extra spending or host execution authority.

## Files and checks

- New `config/opencode/.config/opencode/commands/impeccable.md`: native V2 template, same session/model, no shell expansion.
- New `config/opencode/.config/opencode/skills/impeccable/`: adapted SKILL.md and six compact reference files, pinned provenance, unmodified Apache license and third-party notice. Each derivative file identifies its source and modifications. No scripts, binaries, hooks or executable grants.
- Extend `apps/config-tests/tests/opencode_commands.rs`: preserve existing command contracts, verify the new command plus skill registration/content/description, and restrict the vendored payload to the named static resources.
- Keep existing code-review/simplify definitions, task 38a4e84f, global instructions, package lists/manifests, provider settings and all product source untouched.

## Execution

- [x] Extend regression checks and observe a missing-definition failure before adding the payload.
- [x] Add the command and adapted skill/reference/license/provenance files; review all upstream references for removed executable requirements and design-authority conflicts.
- [x] Run focused native discovery and full offline suite through existing host/resource approvals. No paid model runs or product redesign for testing; disclose untested model behavior.
- [x] Self-review three boundary scenarios: Filament/approved-frame conflict, report-only audit, unsupported engine/live request. These are instruction walkthroughs, not independent agent evaluations.
- [ ] Commit only task files on feature/impeccable-guidance, from fetched origin/main. The old review-command branch and origin/main have identical trees; do not export/reapply its earlier commit.
- [ ] Export only the new commit and request normal reviewed cockpit apply. Verify live tests, source/live/global contents and real command/skill discovery. Stow simulation must exclude unrelated JSON settings, as the earlier review-command deployment established.
- [ ] Report supported/unsupported actions, overlap with installed design skills, pin/update maintenance, exact files/stat/tests, host-facing changes and real deployment status.

## Review and maintenance

Overlaps are intentional guidance overlap with frontend-design, baseline-ui, improve-ui, fixing-accessibility, create-design-md and design-first, not overrides of their IDs or commands. No top-level `/audit`, `/polish` or other aliases are added; `/code-review` and `/simplify` remain independent and are not replaced by design critique or distillation.

Updates are manual: inspect upstream changes at a new immutable revision, adapt only the approved static guidance, preserve notices, rerun discovery and review the host apply. Do not use the upstream installer/updater or silently enable executable features. The normal skill-creator multi-agent/paid evaluation loop is excluded by the no-worker/no-extra-spending constraint; tests establish payload/discovery, not improved design outcomes.

## Execution evidence

- RED receipt `6f485c172433`: `test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out`; missing `/impeccable` definition and skill directory, exit 101.
- After adding the payload, the existing compiled test binary reported `2 passed; 1 failed`: skill API content retained outer newlines. Comparing trimmed bodies fixes the test expectation without changing skill semantics.
- Receipt `a2a9440e277e`: focused `3 passed; 0 failed`; full `make test` failed in `native_discovery_loads_global_commands_and_the_impeccable_skill_without_collisions` with `EOF while parsing a string at line 1 column 327680`. Ancestor project skill discovery had filled the scratch API response with unrelated skills, exposing the native CLI output limit. Set the documented/source-verified project-discovery-disable environment flag only on that scratch server; do not alter live config or permissions. Error previews are bounded rather than dumping unrelated skill contents.
- Final receipt `5ef352baedb6`: focused `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s`; full offline `make test` exit 0, every test group passed. No tests were disabled, no dependencies installed and no models invoked.
- `LICENSE` and `NOTICE.md` match the pinned upstream Git blobs byte-for-byte (`bb3f6d23b1f8025514a62a12b51b47d73e3c9aa9`, `0468271c904ae334cfaf27da6f8df3d5f419a1f0`); regression checks preserve those hashes and restrict the payload to ten named static files.
- Instruction walkthroughs: an approved Filament/frame conflict reports and stops for a design decision; `audit` remains report-only with explicit source/rendered coverage; `live`/engine requests explain the unavailable feature and stop with no install fallback. These were author self-review, not independent model runs or real product redesigns.
- Existing code-review/simplify/global instructions have an empty diff against origin/main. Completion-rule task 38a4e84f and all product source remain untouched. No new workers, shell interpolation, model overrides, hooks, executable global scripts, dependency/provider/security/permission edits or unreviewed host actions were introduced.
- Live application and content/registry verification are pending the new task's separate reviewed cockpit apply; the earlier review-command apply is already complete and must not be replayed.
