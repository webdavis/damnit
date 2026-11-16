use super::ProjectEntry;
use crate::project_hooks::ProjectHookAnswer;

impl ProjectEntry {
    pub(in crate::commands::config) fn with_hooks(
        self,
        now: &ProjectHookAnswer,
        from: &'static str,
    ) -> ProjectEntry {
        ProjectEntry {
            hooks: now.runs(),
            hooks_changed: now.changed(),
            hooks_too_large: matches!(now, ProjectHookAnswer::TooLarge),
            hooks_from: from,
            ..self
        }
    }

    pub(in crate::commands::config) fn hooks_line(&self) -> String {
        let on = match (self.hooks, self.hooks_changed, self.hooks_too_large) {
            (_, true, _) => return "changed since turned on".into(),
            (_, _, true) => return "too large to check".into(),
            (true, ..) => "on",
            (false, ..) => "off",
        };
        format!("{on} (from {})", self.hooks_from)
    }
}
