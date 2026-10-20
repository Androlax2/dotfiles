# Automatic skill activation

Apply this policy in every project, using the skills available in the current
agent session. Do not wait for the user to name a skill.

- Before replying or acting on each request, check the available skill names
  and descriptions. Load matching skills before the response or action they
  govern, including before asking questions when the skill covers discovery.
- Use the agent's native skill-loading tool when available. Otherwise read the
  matching skill's instruction file from a discovered, accessible skill path.
  Do not guess skill IDs, paths, or tool names.
- Follow the loaded instructions, not just their descriptions. Respect their
  scope, sequence, approval gates, and explicit exceptions. Recheck relevance
  when the task changes or reaches a new stage; load newly relevant skills then.
- Use the smallest set that covers the request. Do not load every installed
  skill, force unrelated workflows onto simple questions, or reload instructions
  that are already present and current in the conversation.
- Apply process skills at the stage they govern: brainstorming for creative
  work, writing-plans for multi-step implementation, systematic-debugging for
  unexpected behavior, and test-driven-development before implementation when
  applicable. These examples are not an exhaustive list; check domain skills too.
- If a matching skill cannot be loaded, report the limitation rather than
  silently skipping it or claiming to have used it. If its workflow requires a
  missing tool or capability, stop at that boundary and report what is needed.
- Skills never authorize edits, installs, builds, delegation, git writes, or
  external actions. Preserve the user's permissions, safety rules, and required
  approvals; surface conflicts before proceeding.

## UI Skills CLI fallback

For `ui-skills-root`, check for an installed CLI with `command -v ui-skills`;
do not use `npx` to obtain it. If the CLI is absent or its routing command fails,
report the routing limitation and select the smallest relevant set from the
available skill names and descriptions. Load those skills with the native tool
or a discovered, accessible instruction file, then continue their workflows.
Only the root skill's CLI inspection requirement is waived in this fallback.

Do not retry through another npm cache, download or install a CLI, or change
sandbox permissions to make routing work. This exception does not waive other
skill instructions, design requirements, or approval gates. If a relevant skill
cannot actually be loaded, stop and report the missing capability as usual.

This is a shared instruction policy, not an enforcement mechanism. Each agent
must load it through its supported global-instruction mechanism; merely having
the file on disk does not activate it.
