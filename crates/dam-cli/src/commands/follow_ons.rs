use dam_application::RemovePlan;
use dam_domain::Oid;
use serde::Serialize;

use crate::layout::counted;
use crate::view::displays;

#[derive(Serialize)]
pub(crate) struct FollowOns<'a> {
    #[serde(serialize_with = "displays")]
    descendants: &'a [Oid],
    #[serde(serialize_with = "displays")]
    dependents: &'a [Oid],
    #[serde(serialize_with = "displays")]
    attached: &'a [Oid],
}

impl<'a> FollowOns<'a> {
    pub(crate) fn of(plan: &'a RemovePlan) -> FollowOns<'a> {
        FollowOns {
            descendants: &plan.moved_up,
            dependents: &plan.dependents,
            attached: &plan.attached,
        }
    }

    pub(crate) fn lines(&self) -> Vec<String> {
        let shorts = |v: &[Oid]| v.iter().map(Oid::short).collect::<Vec<_>>().join(", ");
        [
            ("moved up", self.descendants),
            ("no longer depend on it", self.dependents),
            ("still attached to it", self.attached),
        ]
        .into_iter()
        .filter(|(_, oids)| !oids.is_empty())
        .map(|(title, oids)| format!("{title}: {}", shorts(oids)))
        .collect()
    }

    pub(crate) fn counts(&self) -> Option<String> {
        let counts: Vec<String> = [
            (self.descendants, "task moves up", "tasks move up"),
            (
                self.dependents,
                "task stops depending on it",
                "tasks stop depending on it",
            ),
            (self.attached, "task stays attached", "tasks stay attached"),
        ]
        .into_iter()
        .filter(|(oids, _, _)| !oids.is_empty())
        .map(|(oids, one, many)| counted(oids.len(), one, many))
        .collect();
        (!counts.is_empty()).then(|| format!("if added: {}", counts.join(", ")))
    }
}
