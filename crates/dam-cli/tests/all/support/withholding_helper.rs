use std::os::unix::fs::PermissionsExt;

use serde_json::Value;

use super::sandbox::Sandbox;

const HELPER: &str = r#"#!/bin/sh
set -eu
here="$(dirname "$PUSH_LOG")"
while IFS= read -r line; do
  case "$line" in
    *'"capabilities"'*) printf '{"protocol":1,"kinds":["task"],"fields":["subject"],"credentials":["api_token"],"incremental":false}\n' ;;
    *'"pull"'*) cat "$here/pull-answer" ;;
    *'"push"'*)
      printf '%s\n' "$line" >> "$PUSH_LOG"
      if [ -e "$here/answer-next-push" ]; then
        rm "$here/answer-next-push"
        touch "$here/withheld"
        printf '%s' "$line" | grep -o '"oid":"[0-9a-f]*"' | sort -u | cut -d'"' -f4 |
          { grep -v -x -F -f "$here/withheld" || true; } |
          awk 'BEGIN{printf "{\"results\":["} NR>1{printf ","} {printf "{\"oid\":\"%s\",\"ok\":true,\"remote_id\":\"r-%s\",\"why\":null}", $0, $0} END{print "]}"}'
      elif [ -e "$here/refuse-next-push" ]; then
        rm "$here/refuse-next-push"
        printf '%s' "$line" | grep -o '"oid":"[0-9a-f]*"' | sort -u | cut -d'"' -f4 |
          awk 'BEGIN{printf "{\"results\":["} NR>1{printf ","} {printf "{\"oid\":\"%s\",\"ok\":false,\"remote_id\":null,\"why\":\"refused\"}", $0} END{print "]}"}'
      else
        printf '{"results":[]}\n'
      fi ;;
    *) printf '{"error":"unknown request"}\n' ;;
  esac
done
"#;

pub fn listed_task(remote_id: &str, subject: &str) -> String {
    format!(
        r#"{{"oid":"","remote_id":"{remote_id}","kind":"task","subject":"{subject}","body":"","path":"","labels":[],"depends":[],"reminders":[],"recurrence":null,"task":{{"done":false,"priority":4,"due":null,"deadline":null,"event":null}},"event":null}}"#
    )
}

impl Sandbox {
    pub fn install_helper_withholding_answers(&self) {
        let helper = self.dir.path().join("bin/dam-remote-fake");
        std::fs::write(&helper, HELPER).unwrap();
        std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    pub fn answer_the_next_push(&self) {
        std::fs::write(self.dir.path().join("answer-next-push"), "").unwrap();
    }

    pub fn answer_the_next_pull(&self, objects: &[String], removed: &[String]) {
        let removed: Vec<String> = removed.iter().map(|id| format!(r#""{id}""#)).collect();
        let answer = format!(
            r#"{{"objects":[{}],"removed":[{}],"sync":null}}"#,
            objects.join(","),
            removed.join(",")
        );
        std::fs::write(self.dir.path().join("pull-answer"), answer + "\n").unwrap();
    }

    pub fn run(&self, args: &[&str]) {
        let (ok, _, err) = self.dam(args);
        assert!(ok, "{args:?}: {err}");
    }

    pub fn commit_everything(&self) {
        self.run(&["add", "-A"]);
        self.run(&["commit", "-m", "m"]);
    }

    pub fn oid_of(&self, subject: &str) -> String {
        let (ok, out, err) = self.dam(&["ls", "--json"]);
        assert!(ok, "{err}");
        let listed: Value = serde_json::from_str(&out).unwrap();
        let objects = listed["objects"].as_array().unwrap();
        let object = objects.iter().find(|o| o["subject"] == subject).unwrap();
        object["oid"].as_str().unwrap().to_string()
    }
}
