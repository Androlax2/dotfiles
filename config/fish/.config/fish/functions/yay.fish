# AUR builds yield the CPU to interactive work and reclaim memory past 12G instead of pushing the
# whole system into swap. `command yay` bypasses the cap. Keep in sync with yay_capped in the Makefile.
function yay --description 'yay at low CPU weight with a 12G memory cap'
    systemd-run --user --scope --quiet -p CPUWeight=20 -p MemoryHigh=12G yay $argv
end
