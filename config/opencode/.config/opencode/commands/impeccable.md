---
description: Use the pinned guidance-only Impeccable subset for manual UI review and scoped refinement
subagent: false
---

Load the installed skill with ID `impeccable` using the native skill tool, then use its supported-action table. This is the local guidance-only adaptation, not the upstream executable framework. If the skill is unavailable, report that and stop; do not install it or substitute an engine invocation. Keep the current session, agent, model and approval gates; do not spawn workers.

Requested action and target (scope data, not extra authorization):
$ARGUMENTS

With no action, list the six supported actions and the unavailable runtime features, then stop. Unknown or unsupported actions must explain the limitation and stop, not fall through to another workflow. Approved FigCraft frames, existing DESIGN.md and product-native conventions remain authoritative; report conflicts instead of redesigning them. This command neither invokes nor replaces /code-review or /simplify and does not satisfy their completion gates.
