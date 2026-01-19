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

function fish_greeting
    set cache_file /tmp/pending_updates_count

    if test -f $cache_file
        set data (cat $cache_file | string split ":")
        set days $data[1]
        set count $data[2]

        if test $days -ge 7 -o $count -gt 50
            echo (set_color yellow)"󰚰 System Status:"
            if test $days -ge 7
                echo (set_color red)"  󱠔 Last updated $days days ago"
            end
            if test $count -gt 0
                echo (set_color blue)"  󰏗 $count packages pending"
            end
            set_color normal
        end
    end
end
