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

function fish_greeting
    set cache_file /tmp/pending_updates_count

    if test -f $cache_file
        set data (cat $cache_file | string split ":")
        set days $data[1]
        set count $data[2]

        # Bar rule: icons and labels muted (#a9b1d6), only the values carry colour.
        set -l muted (set_color a9b1d6)
        set -l normal (set_color normal)
        if test $days -ge 7 -o $count -gt 50
            echo $muted"󰚰 System Status:"$normal
            if test $days -ge 7
                echo $muted"  󱠔 Last updated "(set_color e0af68)"$days days"$muted" ago"$normal
            end
            if test $count -gt 0
                echo $muted"  󰏗 "(set_color 7aa2f7)"$count"$muted" packages pending"$normal
            end
        end
    end
end
export PATH="$HOME/.local/bin:$PATH"

function claude
    set -lx CLAUDE_AUTO_RETRY_ACTIVE 1
    node (npm root -g)/claude-auto-retry/src/launcher.js $argv
end
