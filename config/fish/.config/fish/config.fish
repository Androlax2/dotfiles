if status is-interactive
    # Commands to run in interactive sessions can go here
end

# PHP
alias sail "bash vendor/bin/sail"

# pnpm
set -gx PNPM_HOME "/home/theo/.local/share/pnpm"
if not string match -q -- $PNPM_HOME $PATH
  set -gx PATH "$PNPM_HOME" $PATH
end
# pnpm end
