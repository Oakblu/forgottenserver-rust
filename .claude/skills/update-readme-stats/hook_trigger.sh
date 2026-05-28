#!/usr/bin/env bash
# PostToolUse hook — fires update_badges.sh after a successful cargo test call.
# Claude Code pipes the tool-use JSON payload to this script's stdin.

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SKILL_DIR="$(cd "$(dirname "$0")" && pwd)"

payload=$(cat)

# Extract the command that was run
command=$(echo "$payload" | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
    print(d.get('tool_input', {}).get('command', ''))
except Exception:
    print('')
" 2>/dev/null)

# Only act on cargo test invocations
if ! echo "$command" | grep -q 'cargo test'; then
    exit 0
fi

# Extract exit code from the tool response
exit_code=$(echo "$payload" | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
    resp = d.get('tool_response', {})
    code = resp.get('exit_code', resp.get('exitCode'))
    if code is None:
        out = str(resp.get('output', '') or resp.get('content', ''))
        code = 1 if ('FAILED' in out or 'error[' in out) else 0
    print(int(code))
except Exception:
    print(1)
" 2>/dev/null)

if [[ "$exit_code" != "0" ]]; then
    exit 0
fi

# Run badge update in background; silence all output so Claude's UI is not polluted
cd "$REPO_ROOT"
bash "$SKILL_DIR/update_badges.sh" >/dev/null 2>&1 &
