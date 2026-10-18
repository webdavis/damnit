#!/bin/sh
set -eu
printf '%s\n' "${DAM_FAKE_API_TOKEN:-}" > "$FAKE_TOKEN_FILE"
remote="$(dirname "$PUSH_LOG")/remote"
upstream='{"oid":"","remote_id":"up-1","kind":"task","subject":"from upstream","body":"","path":"","labels":[],"depends":[],"reminders":[],"recurrence":null,"task":{"done":false,"priority":2,"due":null,"deadline":null,"event":null},"event":null}'
remember() {
  [ -e "$remote.kept" ] || echo "{}" > "$remote.kept"
  printf '%s' "$1" | jq -c --slurpfile kept "${remote}.kept" '
    .mutations[0] as $m | reduce $m.fields[] as $k (($kept[0] // {});
      if $k | IN("done","priority","due","deadline") then .task[$k] = $m.object.task[$k] else .[$k] = $m.object[$k] end)' > "$remote.next"
  mv "$remote.next" "$remote.kept"
}
base='{"oid":"","remote_id":"r1","kind":"task","subject":"","body":"","path":"","labels":[],"depends":[],"reminders":[],"recurrence":null,"task":{"done":false,"priority":2,"due":null,"deadline":null,"event":null},"event":null}'
while IFS= read -r line; do
  case "$line" in
    *'"capabilities"'*) printf '{"protocol":1,"kinds":["task"],"fields":["subject","body","path","labels","done","priority","due","deadline"],"credentials":["api_token"],"incremental":false}\n' ;;
    *'"pull"'*)
      objects="$upstream"
      if [ -e "$remote.kept" ]; then
        objects="$objects,$(printf '%s' "$base" | jq -c --slurpfile kept "$remote.kept" '. * $kept[0]')"
      fi
      printf '{"objects":[%s],"removed":[],"sync":null}\n' "$objects" ;;
    *'"push"'*)
      printf '%s\n' "$line" >> "$PUSH_LOG"
      if [ -e "$FAIL_NEXT_PUSH" ]; then rm "$FAIL_NEXT_PUSH"; ok=false; why='"temporary failure"'; else ok=true; why=null; remember "$line"; fi
      printf '{"results":[' ; printf '%s' "$line" | tr ',' '\n' | grep -o '"oid":"[0-9a-f]*"' | sort -u | awk -v ok="$ok" -v why="$why" 'NR>1{printf ","}{printf "{%s,\"ok\":%s,\"remote_id\":\"r%d\",\"why\":%s}", $0, ok, NR, why}' ; printf ']}\n' ;;
    *) printf '{"error":"unknown request"}\n' ;;
  esac
done
