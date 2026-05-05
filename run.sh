#!/usr/bin/env bash

export GREETD_SOCK=/tmp/greetd.sock
export XDG_SESSION_TYPE=wayland
export XDG_CURRENT_DESKTOP=Hyprland

./target/release/regreet
