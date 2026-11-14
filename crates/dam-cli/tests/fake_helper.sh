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
      if [ -e "${HANG_NEXT_PUSH:-}" ]; then
        answer="$(cat "$HANG_NEXT_PUSH")"; rm "$HANG_NEXT_PUSH"
        first="$(printf '%s' "$line" | jq -c '.mutations[0].oid')"
        if [ "$answer" = partial ]; then printf '{"early_answer":{"oid":%s,"ok":true,"remote_id":"r1"}}\n' "$first"; fi
        touch "$HANG_NEXT_PUSH.hung"; exec sleep 30
      fi
      if [ -e "${RATE_LIMIT_NEXT_PUSH:-}" ]; then
        answer="$(cat "$RATE_LIMIT_NEXT_PUSH")"; rm "$RATE_LIMIT_NEXT_PUSH"
        stop='"fake is rate limiting this account; retry after 42s"'
        if [ "$answer" = error ]; then printf '{"error":%s}\n' "$stop"; continue; fi
        first="$(printf '%s' "$line" | jq -c '.mutations[0].oid')"
        printf '{"results":[{"oid":%s,"ok":true,"remote_id":"r1"}],"stopped":%s}\n' "$first" "$stop"; continue
      fi
      refused=none
      if [ -e "$FAIL_NEXT_PUSH" ]; then rm "$FAIL_NEXT_PUSH"; ok=false; why='"temporary failure"'
      elif [ -e "${FAIL_FIRST_OF_NEXT_PUSH:-}" ]; then rm "$FAIL_FIRST_OF_NEXT_PUSH"; ok=true; why=null; refused="$(printf '%s' "$line" | jq -r '.mutations[0].oid')"
      else ok=true; why=null; remember "$line"; fi
      printf '{"results":[' ; printf '%s' "$line" | tr ',' '\n' | grep -o '"oid":"[0-9a-f]*"' | sort -u | awk -v ok="$ok" -v why="$why" -v refused="\"oid\":\"$refused\"" 'NR>1{printf ","}{o=ok; w=why; if ($0 == refused) {o="false"; w="\"temporary failure\""} printf "{%s,\"ok\":%s,\"remote_id\":\"r%d\",\"why\":%s}", $0, o, NR, w}' ; printf ']}\n' ;;
    *) printf '{"error":"unknown request"}\n' ;;
  esac
done
