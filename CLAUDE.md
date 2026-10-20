# Dotfiles

## Git authorization

Claude may create feature branches, stage and commit completed, requested dotfiles
changes without asking again in chat. Never commit directly to the default branch.
Pushing and opening pull requests still require an explicit request.

## Applying completed work

Finishing requested dotfiles changes includes applying them to the live cockpit,
without asking again in chat. Export the commits with
`git format-patch origin/main -o .outgoing`, then use `ask-to-run` to run
`claude-sessions apply-dotfiles <clone-root>`. Keep the host-side review and approval;
never bypass it. After applying, run `make -C ~/dotfiles cockpit test` and, when
`apps/` changed, `claude-sessions reload` through `ask-to-run`.
