#!/bin/bash
# Claude Code toast notification for Windows (Git Bash in Windows Terminal)
# Reads hook JSON from stdin to extract project name, message, etc.
# Only shows notification if the terminal window is NOT focused.
# Clicking the notification brings the terminal window to foreground.

TITLE="${1:-Claude Code}"

# Read stdin (hook JSON) if available
INPUT=""
if [ ! -t 0 ]; then
  INPUT=$(cat)
fi

# Single PowerShell call: focus check + JSON parse + toast
powershell.exe -NoProfile -ExecutionPolicy Bypass -File \
  "$(cygpath -w ~/bin/claude-notify.ps1)" \
  -Title "$TITLE" \
  -FallbackMessage "${2:-}" \
  -JsonInput "$INPUT" &

exit 0
