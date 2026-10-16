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

# Route AI CLI commands and Claude auto-retry launches through Greywall.
set -gx PATH "$HOME/.local/bin/greywall-agents" $PATH

function claude
    set -lx CLAUDE_AUTO_RETRY_ACTIVE 1
    # An interactive `claude` from any terminal opens in the session manager on its workspace
    # (~/.local/bin/claude-sessions). The manager's own viewer pane (CLAUDE_SESSIONS_INSIDE),
    # print mode and the informational flags run claude right here.
    if test -z "$CLAUDE_SESSIONS_INSIDE"; and isatty stdin; and isatty stdout
        and not contains -- -p $argv; and not contains -- --print $argv
        and not contains -- --help $argv; and not contains -- --version $argv
        claude-sessions new (pwd) $argv; and claude-sessions show
        return
    end
    node (npm root -g)/claude-auto-retry/src/launcher.js $argv
end

# fzf colours (Tokyo Night, from folke/tokyonight.nvim extras/fzf)
source ~/.config/fzf/tokyonight_night.sh
