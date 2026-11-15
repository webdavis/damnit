use dam_application::HookEvent;
use dam_domain::{Object, Oid};
use serde::Serialize;

use crate::context::Context;
use crate::error::CliError;
use crate::view::{Human, View, display};

#[derive(Serialize)]
pub(crate) struct Hooked<V> {
    #[serde(flatten)]
    view: V,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    changed_by_hook: Vec<HookChange>,
}

impl<V: View> View for Hooked<V> {
    fn human(&self) -> Human {
        self.view.human()
    }

    fn warnings(&self) -> Vec<String> {
        self.view.warnings()
    }

    fn notices(&self) -> Vec<String> {
        let mut notices = self.view.notices();
        notices.extend(self.changed_by_hook.iter().map(HookChange::note));
        notices
    }
}

pub(crate) fn hooked<V>(ctx: &Context, view: V) -> Hooked<V> {
    Hooked {
        view,
        changed_by_hook: HookChange::every(ctx),
    }
}

pub(crate) fn as_it_is_now(ctx: &Context, object: &Object) -> Result<Object, CliError> {
    Ok(ctx
        .store
        .get(object.oid())?
        .unwrap_or_else(|| object.clone()))
}

#[derive(Serialize)]
pub(crate) struct HookChange {
    #[serde(serialize_with = "display")]
    oid: Oid,
    hook: &'static str,
}

impl HookChange {
    fn every(ctx: &Context) -> Vec<HookChange> {
        let changes = ctx.changed_by_hook.all().into_iter();
        changes
            .map(|(event, oid): (HookEvent, Oid)| HookChange {
                oid,
                hook: event.as_str(),
            })
            .collect()
    }

    fn note(&self) -> String {
        format!("hook {} changed {}", self.hook, self.oid.short())
    }
}
