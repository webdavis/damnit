use dam_domain::Grant;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct GrantView {
    allow: Vec<&'static str>,
    paths: Option<Vec<String>>,
    labels: Option<Vec<String>>,
    own: bool,
    remotes: Option<Vec<String>>,
}

impl GrantView {
    pub(super) fn of(grant: &Grant) -> GrantView {
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
}
