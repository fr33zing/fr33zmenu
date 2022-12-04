#!/bin/sh

fr33zmenu ~/.config/fr33zmenu/menu.toml \
    --exec-with "nohup hyprctl dispatch exec" \
    --transient
