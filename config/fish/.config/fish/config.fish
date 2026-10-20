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
    # An interactive `claude` from any terminal asks where to run: Enter (or m) opens it in the
    # session manager on its workspace (~/.local/bin/claude-sessions), t runs it in this terminal,
    # any other key (or Ctrl-C) cancels. The manager's own viewer pane (CLAUDE_SESSIONS_INSIDE),
    # print mode and the informational flags run claude right here without asking.
    if test -z "$CLAUDE_SESSIONS_INSIDE"; and isatty stdin; and isatty stdout
        and not contains -- -p $argv; and not contains -- --print $argv
        and not contains -- --help $argv; and not contains -- --version $argv
        set -l prompt (set_color --bold blue)'claude'(set_color normal)'  '(set_color --bold)'⏎'(set_color normal)' session manager  '(set_color --bold)'t'(set_color normal)' this terminal  '
        read -l -n 1 -P $prompt choice; or return 130
        switch $choice
            case '' m M
                claude-sessions --agent claude new (pwd) $argv; and claude-sessions show
                return
            case t T
                # Run right here, below. The marker reaches the session (the retry wrapper carries the
                # environment): its card says "terminal", and the cockpit's restart-all, restore,
                # queue, error hand-off, ask-to-run and ask-to-guide leave it alone.
                set -fx CLAUDE_SESSIONS_TERMINAL 1
            case '*'
                echo "claude: cancelled"
                return 1
        end
    end
    node (npm root -g)/claude-auto-retry/src/launcher.js $argv
end

# Like claude: a bare interactive `opencode` asks where to run, the session manager (Enter or m,
# its card like a Claude session's), the existing cockpit in this terminal (r), or attaches
# the same managed process here (t). With arguments (run, serve, auth, a project) and in the
# manager's own launches it runs right here, through greywall-agents/opencode.
function opencode
    if test -z "$CLAUDE_SESSIONS_INSIDE"; and isatty stdin; and isatty stdout; and test (count $argv) -eq 0
        set -l prompt (set_color --bold blue)'opencode'(set_color normal)'  '(set_color --bold)'⏎'(set_color normal)' session manager  '(set_color --bold)'r'(set_color normal)' resume session  '(set_color --bold)'t'(set_color normal)' this terminal  '
        read -l -n 1 -P $prompt choice; or return 130
        switch $choice
            case '' m M
                claude-sessions --agent opencode new (pwd); and claude-sessions show
                return
            case r R
                claude-sessions attach
                return
            case t T
                claude-sessions --agent opencode terminal (pwd)
                return
            case '*'
                echo "opencode: cancelled"
                return 1
        end
    end
    command opencode $argv
end

# fzf colours (Tokyo Night, from folke/tokyonight.nvim extras/fzf)
source ~/.config/fzf/tokyonight_night.sh
