#!/usr/bin/env bash

POMODORO_APP="waybar-module-pomodoro"

# The action to perform on a single click
SINGLE_CLICK_ACTION="$POMODORO_APP toggle"

# The action to perform on a double click
DOUBLE_CLICK_ACTION="$POMODORO_APP skip"

# --- Script Logic ---
TMP_FILE="/tmp/waybar_double_click_lock"
THRESHOLD=300 # Milliseconds

if [ -f "$TMP_FILE" ]; then
    LAST_TIME=$(cat "$TMP_FILE")
    TIME_DIFF=$((CURRENT_TIME - LAST_TIME))

    if [ "$TIME_DIFF" -lt "$THRESHOLD" ]; then
        # Double click detected
        eval "$DOUBLE_CLICK_ACTION"
        rm "$TMP_FILE" # Reset for the next click sequence
        exit 0
    fi
fi

# Not a double click, so save the time and wait
echo "$CURRENT_TIME" > "$TMP_FILE"
sleep 0.2 # Wait for a potential second click

# If the file still exists, it means no second click occurred
if [ -f "$TMP_FILE" ]; then
    if [ "$(cat $TMP_FILE)" == "$CURRENT_TIME" ]; then
        # Single click action
        eval "$SINGLE_CLICK_ACTION"
        rm "$TMP_FILE"
    fi
fi
