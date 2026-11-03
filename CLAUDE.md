# Dotfiles

## Git authorization

Claude may create feature branches, stage and commit completed, requested dotfiles
changes without asking again in chat. Never commit directly to the default branch.
Pushing and opening pull requests still require an explicit request.

## Tests

The Python tests in `tools/tests` run on the host only: the disk and cache reports walk every path
from `/`, which a sandbox cannot open (`Permission denied: '/'`).
