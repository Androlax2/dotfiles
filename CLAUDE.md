# Dotfiles

## Git authorization

Claude may create feature branches, stage and commit completed, requested dotfiles
changes without asking again in chat. Never commit directly to the default branch.
Pushing and opening pull requests still require an explicit request.

## Applying completed work

Finishing requested dotfiles changes includes applying them to the live cockpit,
without asking again in chat. Export the commits with
`git format-patch origin/main -o .outgoing`, then use `ask-to-run` to run
`cockpit apply <clone-root>`. Keep the host-side review and approval;
never bypass it. After applying, run `make -C ~/dotfiles test` through `ask-to-run`.

The Python tests in `tools/tests` run on the host only (`ask-to-run`): the disk and cache reports
walk every path from `/`, which the sandbox cannot open (`Permission denied: '/'`).

The Agent Cockpit and vitals are repositories of their own (`~/agent-cockpit`, `~/vitals`, worked
on in `~/IdeaProjects/agent-cockpit` and `~/IdeaProjects/vitals`); work on them belongs to a
session there, filed with `cockpit-todo propose … --project <name>`.

## Todo trailers

End a commit message with one `Todo: <id>` line per dotfiles todo item the commit fixes
(8 hex characters, as `cockpit-todo list` shows). `cockpit apply` marks those items done once
Theo approved the commits.
