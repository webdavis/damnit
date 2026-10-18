#!/bin/sh
set -eu
printf '%s\n' "${DAM_FAKE_API_TOKEN:-}" > "$FAKE_TOKEN_FILE"
while IFS= read -r line; do
  case "$line" in
    *'"capabilities"'*) printf '{"protocol":1,"kinds":["task"],"fields":["subject","body","path","labels","done","priority","due","deadline"],"credentials":["api_token"],"incremental":false}\n' ;;
    *'"pull"'*) printf '{"objects":[{"oid":"","remote_id":"up-1","kind":"task","subject":"from upstream","body":"","path":"","labels":[],"depends":[],"reminders":[],"recurrence":null,"task":{"done":false,"priority":2,"due":null,"deadline":null,"event":null},"event":null}],"removed":[],"sync":null}\n' ;;
    *'"push"'*)
      printf '%s\n' "$line" >> "$PUSH_LOG"
      if [ -e "$FAIL_NEXT_PUSH" ]; then rm "$FAIL_NEXT_PUSH"; ok=false; why='"temporary failure"'; else ok=true; why=null; fi
      printf '{"results":[' ; printf '%s' "$line" | tr ',' '\n' | grep -o '"oid":"[0-9a-f]*"' | sort -u | awk -v ok="$ok" -v why="$why" 'NR>1{printf ","}{printf "{%s,\"ok\":%s,\"remote_id\":\"r%d\",\"why\":%s}", $0, ok, NR, why}' ; printf ']}\n' ;;
    *) printf '{"error":"unknown request"}\n' ;;
  esac
done
