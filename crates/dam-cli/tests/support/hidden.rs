use std::process::Output;

use super::sandbox::Sandbox;
use super::tokens::grant_to;

pub struct Hidden {
    pub sb: Sandbox,
    pub held: String,
    pub report: String,
}

impl Hidden {
    pub fn new() -> Hidden {
        let sb = Sandbox::new();
        let held = grant_to(
            &sb,
            "tool",
            1,
            "[[token.tool.grant]]\n\
             allow = [\"task:read\", \"task:change\", \"task:complete\", \"event:read\", \"event:change\"]\n\
             paths = [\"work\"]\n",
        );
        let report = full_oid(&sb, &sb.new_object(&["report", "--path", "work"]));
        Hidden { sb, held, report }
    }

    pub fn task(&self, subject: &str, path: &str) -> String {
        full_oid(&self.sb, &self.sb.new_object(&[subject, "--path", path]))
    }

    pub fn event(&self, subject: &str, path: &str) -> String {
        let short = self.sb.new_object(&[
            "--event",
            subject,
            "--path",
            path,
            "--start",
            "2026-09-25T14:00",
            "--end",
            "2026-09-25T14:30",
        ]);
        full_oid(&self.sb, &short)
    }

    pub fn each_hidden_by_oid_and_prefix(&self, check: impl Fn(&str, &str)) {
        for kind in ["task", "event"] {
            for prefix in [None, Some(7)] {
                let hidden = match kind {
                    "task" => self.task("diary", "home"),
                    _ => self.event("dentist", "home"),
                };
                let named = prefix.map_or(hidden.as_str(), |n| &hidden[..n]);
                check(&hidden, named);
            }
        }
    }

    pub fn owner(&self, args: &[&str]) {
        let (ok, _, err) = self.sb.dam(args);
        assert!(ok, "{args:?}: {err}");
    }

    pub fn holding(&self, args: &[&str]) -> Output {
        self.sb.output_holding(&self.held, args)
    }

    pub fn answers_alike_once_removed(&self, hidden: &str, args: &[&str]) -> Output {
        let while_hidden = self.holding(args);
        self.owner(&["rm", hidden]);
        let once_missing = self.holding(args);
        assert_eq!(
            (
                while_hidden.status.code(),
                text(&while_hidden.stdout),
                text(&while_hidden.stderr)
            ),
            (
                once_missing.status.code(),
                text(&once_missing.stdout),
                text(&once_missing.stderr)
            ),
            "{args:?}"
        );
        while_hidden
    }
}

pub fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

pub fn full_oid(sb: &Sandbox, short: &str) -> String {
    let (ok, out, err) = sb.dam(&["show", short, "--json", "--no-pull"]);
    assert!(ok, "{err}");
    serde_json::from_str::<serde_json::Value>(&out).unwrap()["oid"]
        .as_str()
        .unwrap()
        .to_string()
}
