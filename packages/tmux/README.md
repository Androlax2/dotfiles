# Local tmux border-junction fix

`top-border-junction.patch` clips the focused top border to its pane when
`@pane-border-clip-top` is `on`. The locally patched tmux 3.8-1 keeps the
official package version, so pacman can silently replace it on an upgrade.

`cockpit doctor` hashes the `tmux` executable on PATH and compares it
with `config/tmux/.config/tmux/border-junction.sha256` (stowed under
`~/.config/tmux/`). The baseline records the installed patched binary, whose
`@pane-border-clip-top` marker was verified when recording it. A replacement,
unreadable executable, or missing/invalid baseline fails the check. The existing
weekly health check runs doctor and raises its normal banner on failure; this
is not an immediate pacman notification.

After an upgrade, review and reapply the patch to the new upstream source,
run `python3 packages/tmux/test-border-junction.py /path/to/tmux/source`, then
install the reviewed package and verify the border in the cockpit. Only then
record the new executable's SHA-256 in the baseline. Do not refresh the hash
just to silence the warning: a matching version alone does not prove the patch
is present. This check covers the executable on disk, not an already-running
tmux server, and never rebuilds or installs anything automatically.
