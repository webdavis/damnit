use dam_domain::Grant;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct GrantView {
    allow: Vec<&'static str>,
    paths: Option<Vec<String>>,
    labels: Option<Vec<String>>,
    own: bool,
    remotes: Option<Vec<String>>,
}

impl GrantView {
    pub(crate) fn of(grant: &Grant) -> GrantView {
        GrantView {
            allow: grant.allow.iter().map(|p| p.as_str()).collect(),
            paths: grant
                .paths
                .as_ref()
                .map(|paths| paths.iter().map(|p| p.to_string()).collect()),
            labels: grant.labels.as_ref().map(|l| l.iter().cloned().collect()),
            own: grant.own,
            remotes: grant.remotes.clone(),
        }
    }

    pub(crate) fn text(&self) -> String {
        let mut text = self.allow.join(", ");
        let narrowing = [
            ("paths", &self.paths),
            ("labels", &self.labels),
            ("remotes", &self.remotes),
        ];
        for (key, values) in narrowing {
            if let Some(values) = values {
                text.push_str(&format!(" {key} {}", values.join(", ")));
            }
        }
        if self.own {
            text.push_str(" own");
        }
        text
    }
}
