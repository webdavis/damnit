#!/bin/sh
set -eu
event() {
  printf '{"oid":"","remote_id":"%s","kind":"event","subject":"%s","body":"","path":"%s","labels":[],"depends":[],"reminders":[],"recurrence":null,"task":null,"event":{"start":"%s","end":"%s","status":"confirmed","transparency":"busy","visibility":"default","event_type":"default","attachments":[]%s}}' "$@"
}
people='"attendees":[{"email":"ada@example.com","response":"accepted","self":true},{"email":"grace@example.com","response":"tentative"}],"organizer":{"email":"grace@example.com","name":"Grace"}'
meeting="$(event cal-1 'design review' work '2026-09-25T14:00:00+00:00[UTC]' '2026-09-25T15:00:00+00:00[UTC]' ",$people,\"location\":\"Room 4\",\"conference\":{\"provider\":\"meet\",\"url\":\"https://meet.example.com/abc\"}")"
alone="$(event cal-2 'focus block' work '2026-09-25T09:00:00+00:00[UTC]' '2026-09-25T10:00:00+00:00[UTC]' '')"
hidden="$(event cal-3 'dentist' home '2026-09-25T16:00:00+00:00[UTC]' '2026-09-25T17:00:00+00:00[UTC]' ",$people")"
while IFS= read -r line; do
  case "$line" in
    *'"capabilities"'*) printf '{"protocol":1,"kinds":["event"],"fields":["subject","body","path","start","end","location","attendees","status","transparency","visibility","event_type","organizer","conference","attachments"],"credentials":["api_token"],"incremental":false}\n' ;;
    *'"pull"'*) printf '{"objects":[%s,%s,%s],"removed":[],"sync":null}\n' "$meeting" "$alone" "$hidden" ;;
    *) printf '{"error":"unknown request"}\n' ;;
  esac
done
