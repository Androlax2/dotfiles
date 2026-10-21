# vitals

## Git authorization

Claude may create feature branches, stage and commit completed, requested vitals changes without
asking again in chat. Never commit directly to the default branch. Pushing and opening pull
requests still require an explicit request.

## Applying completed work

`~/vitals` is the running copy the host runs (its collector every minute, its root timer); no
session works in it. Sessions work in `~/IdeaProjects/vitals` or its parallel clones. Finishing
requested changes includes applying them, without asking again in chat: export the commits with
`git format-patch origin/main -o .outgoing`, then use `ask-to-run` to run
`cockpit apply <clone-root>`. Keep the host-side review and approval; never bypass it. The
apply runs `make after-apply` (relink, rebuild); then run `make -C ~/vitals test` through
`ask-to-run`.

## Language

The TUI and the alerts are in French; code, comments and commits in English.

## Todo trailers

End a commit message with one `Todo: <id>` line per todo item the commit fixes (8 hex
characters, as `claude-todo list` shows). `cockpit apply` marks those items done once Theo
approved the commits.
