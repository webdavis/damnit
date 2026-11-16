use dam_application::PushPreview;
use serde::Serialize;

use crate::view::{Human, View};
use rows::{HeldRow, MutationRow, SkippedRow, skipped};

#[derive(Serialize)]
pub(super) struct PushPreviewView<'a> {
    preview: bool,
    remotes: Vec<RemotePlan<'a>>,
}

#[derive(Serialize)]
struct RemotePlan<'a> {
    remote: &'a str,
    known: bool,
    mutations: Vec<MutationRow<'a>>,
    held: Vec<HeldRow<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conflicted: Option<Vec<String>>,
    skipped: Vec<SkippedRow<'a>>,
}

impl<'a> PushPreviewView<'a> {
    pub(super) fn of(previews: &'a [PushPreview]) -> PushPreviewView<'a> {
        PushPreviewView {
            preview: true,
            remotes: previews.iter().map(RemotePlan::of).collect(),
        }
    }
}

impl<'a> RemotePlan<'a> {
    fn of(preview: &'a PushPreview) -> RemotePlan<'a> {
        let empty = RemotePlan {
            remote: &preview.remote.0,
            known: false,
            mutations: Vec::new(),
            held: Vec::new(),
            conflicted: None,
            skipped: Vec::new(),
        };
        let Some(plan) = &preview.plan else {
            return empty;
        };
        RemotePlan {
            known: true,
            mutations: plan.mutations.iter().map(MutationRow::of).collect(),
            held: plan
                .held
                .iter()
                .map(|hold| HeldRow {
                    oid: &hold.oid,
                    why: &hold.why,
                })
                .collect(),
            conflicted: plan
                .conflicted
                .as_ref()
                .map(|oids| oids.iter().map(ToString::to_string).collect()),
            skipped: skipped(plan),
            ..empty
        }
    }

    fn lines(&self) -> Vec<String> {
        if !self.known {
            return vec![format!(
                "{}: not known until dam pull {} has run",
                self.remote, self.remote
            )];
        }
        let mut lines = vec![format!("{}:", self.remote)];
        lines.extend(self.mutations.iter().map(|m| {
            let into = m.container.as_deref().map(|c| format!("  in {c}"));
            format!(
                "  {:<7} {}  {}{}",
                m.op,
                m.oid.short(),
                m.subject.unwrap_or(""),
                into.unwrap_or_default()
            )
        }));
        lines.extend(
            self.held
                .iter()
                .map(|h| format!("  held    {}  {}", h.oid.short(), h.why)),
        );
        lines.extend(
            self.skipped
                .iter()
                .map(|s| format!("  skipped {}  {}", s.oid.short(), s.because)),
        );
        lines
    }
}

impl View for PushPreviewView<'_> {
    fn human(&self) -> Human {
        let mut lines: Vec<String> = self.remotes.iter().flat_map(RemotePlan::lines).collect();
        let total = |count: fn(&RemotePlan) -> usize| self.remotes.iter().map(count).sum::<usize>();
        lines.push(format!(
            "preview: {} to send, {} held, {} skipped, nothing sent",
            total(|r| r.mutations.len()),
            total(|r| r.held.len()),
            total(|r| r.skipped.len()),
        ));
        Human::Lines(lines)
    }
}

mod rows;
