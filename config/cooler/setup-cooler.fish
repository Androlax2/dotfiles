#!/usr/bin/env fish
# Sets up the Frozen Warframe Pro LCD: plays one Eros video on loop with live
# CPU/GPU temperatures drawn on top.
#
# Usage:
#   fish setup-cooler.fish            # default scene (night watch)
#   fish setup-cooler.fish peeking    # any scene, matched by name
#   fish setup-cooler.fish playlist   # all four favourites, cross-faded
#   fish setup-cooler.fish --list     # show available scenes

set cooler_device 87ad:70db
set cooler_dir (status dirname)
set default_scene night-watch
set overlay_color '#28BEFF'
set overlay_size 20
set overlay_mask $cooler_dir/overlay-mask-320.png
# Values sit right of the CPU/GPU labels drawn in the mask's pills.
set value_x 188
set cpu_value_y 22
set gpu_value_y 298

function list_scenes
    for video in $cooler_dir/eros-*-320.mp4
        string replace -r '^eros-(\d+-)?(.+)-320\.mp4$' '$2' (path basename $video)
    end
end

function find_scene_video --argument-names scene_name
    set matching_videos $cooler_dir/eros-*$scene_name*-320.mp4
    if test (count $matching_videos) -eq 0
        echo "No scene matches '$scene_name'. Available:" >&2
        list_scenes >&2
        return 1
    end
    echo $matching_videos[1]
end

function play_scene --argument-names scene_video
    trcc display load-video $cooler_device $scene_video
    and trcc display loop-video $cooler_device on
end

function show_label_mask
    trcc display apply-mask $cooler_device $overlay_mask
    and trcc display mask-visible $cooler_device on
end

function add_temperature_overlay --argument-names element_id metric position_y
    # Drop a previous copy so re-running the script doesn't stack duplicates.
    trcc display overlay-delete $cooler_device $element_id >/dev/null 2>&1
    trcc display overlay-add $cooler_device metric \
        --id $element_id \
        --metric $metric \
        --x $value_x --y $position_y \
        --color $overlay_color --size $overlay_size \
        --bold
end

function show_temperatures
    trcc display overlay $cooler_device on
    and add_temperature_overlay cpu-temp cpu:temp $cpu_value_y
    and add_temperature_overlay gpu-temp gpu:primary:temp $gpu_value_y
end

if test "$argv[1]" = --list
    list_scenes
    exit 0
end

set requested_scene $default_scene
if set -q argv[1]
    set requested_scene $argv[1]
end

set scene_video (find_scene_video $requested_scene)
or exit 1

play_scene $scene_video
and show_label_mask
and show_temperatures
