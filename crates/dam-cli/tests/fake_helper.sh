#!/bin/sh
# Answers capabilities, pulls one fixed task, echoes every push as ok.
# Records the token it was given so the test can assert delivery.
set -eu
printf '%s\n' "${DAM_FAKE_API_TOKEN:-}" > "$FAKE_TOKEN_FILE"
while IFS= read -r line; do
  case "$line" in
    *'"capabilities"'*) printf '{"protocol":1,"kinds":["task"],"fields":["subject","body","path","labels","done","priority","due","deadline"],"credentials":["api_token"],"incremental":false}\n' ;;
    *'"pull"'*) printf '{"objects":[{"oid":"","remote_id":"up-1","kind":"task","subject":"from upstream","body":"","path":"","labels":[],"depends":[],"reminders":[],"recurrence":null,"task":{"done":false,"priority":2,"due":null,"deadline":null,"event":null},"event":null}],"removed":[],"sync":null}\n' ;;
    *'"push"'*) printf '{"results":[' ; printf '%s' "$line" | tr ',' '\n' | grep -o '"oid":"[0-9a-f]*"' | sort -u | awk 'NR>1{printf ","}{printf "{%s,\"ok\":true,\"remote_id\":\"r%d\",\"why\":null}", $0, NR}' ; printf ']}\n' ;;
    *) printf '{"error":"unknown request"}\n' ;;
  esac
done
