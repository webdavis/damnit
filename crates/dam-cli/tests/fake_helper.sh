#!/bin/sh
set -eu
printf '%s\n' "${DAM_FAKE_API_TOKEN:-}" > "$FAKE_TOKEN_FILE"
remote="$(dirname "$PUSH_LOG")/remote"
upstream='{"oid":"","remote_id":"up-1","kind":"task","subject":"from upstream","body":"","path":"","labels":[],"depends":[],"reminders":[],"recurrence":null,"task":{"done":false,"priority":2,"due":null,"deadline":null,"event":null},"event":null}'
remember() {
  fields=$(printf '%s' "$1" | grep -o '"fields":\[[^]]*\]')
  case "$fields" in *'"subject"'*) printf '%s' "$1" | grep -o '"subject":"[^"]*"' | head -1 | cut -d: -f2- > "$remote.subject" ;; esac
  case "$fields" in *'"done"'*) printf '%s' "$1" | grep -o '"done":[a-z]*' | head -1 | cut -d: -f2 > "$remote.done" ;; esac
}
while IFS= read -r line; do
  case "$line" in
    *'"capabilities"'*) printf '{"protocol":1,"kinds":["task"],"fields":["subject","body","path","labels","done","priority","due","deadline"],"credentials":["api_token"],"incremental":false}\n' ;;
    *'"pull"'*)
      objects="$upstream"
      if [ -e "$remote.subject" ]; then
        objects="$objects,{\"oid\":\"\",\"remote_id\":\"r1\",\"kind\":\"task\",\"subject\":$(cat "$remote.subject"),\"body\":\"\",\"path\":\"\",\"labels\":[],\"depends\":[],\"reminders\":[],\"recurrence\":null,\"task\":{\"done\":$(cat "$remote.done" 2>/dev/null || echo false),\"priority\":2,\"due\":null,\"deadline\":null,\"event\":null},\"event\":null}"
      fi
      printf '{"objects":[%s],"removed":[],"sync":null}\n' "$objects" ;;
    *'"push"'*)
      printf '%s\n' "$line" >> "$PUSH_LOG"
      if [ -e "$FAIL_NEXT_PUSH" ]; then rm "$FAIL_NEXT_PUSH"; ok=false; why='"temporary failure"'; else ok=true; why=null; remember "$line"; fi
      printf '{"results":[' ; printf '%s' "$line" | tr ',' '\n' | grep -o '"oid":"[0-9a-f]*"' | sort -u | awk -v ok="$ok" -v why="$why" 'NR>1{printf ","}{printf "{%s,\"ok\":%s,\"remote_id\":\"r%d\",\"why\":%s}", $0, ok, NR, why}' ; printf ']}\n' ;;
    *) printf '{"error":"unknown request"}\n' ;;
  esac
done
