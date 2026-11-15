use super::Refusal;

impl Refusal {
    pub fn name(&self) -> &'static str {
        match self {
            Refusal::Blocked { .. } => "blocked",
            Refusal::Cycle { .. } => "cycle",
            Refusal::Labels(_) => "exclusive_label",
            Refusal::UnknownCategory(_) => "unknown_category",
            Refusal::NotACategoryValue { .. } => "not_a_category_value",
            Refusal::NoSuchObject(_) => "no_such_object",
            Refusal::NoWorkingObject(_) => "no_working_object",
            Refusal::AmbiguousPrefix { .. } => "ambiguous_prefix",
            Refusal::NoSuchRemote(_) => "no_such_remote",
            Refusal::RemoteExists(_) => "remote_exists",
            Refusal::NotATask(_) => "not_a_task",
            Refusal::NotAnEvent(_) => "not_an_event",
            Refusal::NotCompleted(_) => "not_completed",
            Refusal::NotCommitted(_) => "not_committed",
            Refusal::MoveInsideItself(_) => "move_inside_itself",
            Refusal::SubjectNamesNoFolder(_) => "subject_names_no_folder",
            Refusal::NothingToCommit => "nothing_to_commit",
            Refusal::NeedsAnAnswer => "needs_an_answer",
            Refusal::NeedsAnEditor => "needs_an_editor",
            Refusal::MissingCredential { .. } => "missing_credential",
            Refusal::StaleRemote { .. } => "stale_remote",
            Refusal::UnreadableRecurrence { .. } => "unreadable_recurrence",
            Refusal::NoConflict(_) => "no_conflict",
            Refusal::Token(refusal) => refusal.name(),
            Refusal::NotPermitted(_) => "not_permitted",
            Refusal::NotOpenToTokens(_) => "not_open_to_tokens",
            Refusal::FocusFull { .. } => "focus_full",
            Refusal::TokenExists(_) => "token_exists",
            Refusal::NotGrantable(_) => "not_grantable",
            Refusal::NoSuchToken(_) => "no_such_token",
            Refusal::NoSuchSetting(_) => "no_such_setting",
            Refusal::HookRefused(_) => "hook_refused",
            Refusal::StartedByHook(_) => "started_by_hook",
            Refusal::Item(refusal) => refusal.name(),
            Refusal::PulledCommit(_) => "pulled_commit",
            Refusal::NotInCommit { .. } => "not_in_commit",
        }
    }
}
