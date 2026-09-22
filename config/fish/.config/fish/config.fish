if status is-interactive
    # Commands to run in interactive sessions can go here
end

# PHP
alias sail "bash vendor/bin/sail"

# pnpm
set -gx PNPM_HOME "/home/theo/.local/share/pnpm"
if not string match -q -- "$PNPM_HOME/bin" $PATH
    set -gx PATH "$PNPM_HOME/bin" $PATH
end
# pnpm end

# Pending updates are shown in Waybar (custom/updates); no greeting in the terminal.
function fish_greeting
end
export PATH="$HOME/.local/bin:$PATH"

function claude
    set -lx CLAUDE_AUTO_RETRY_ACTIVE 1
    node (npm root -g)/claude-auto-retry/src/launcher.js $argv
end
