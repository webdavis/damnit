use std::fmt;

use dam_domain::CommitId;

pub(super) fn pulled_commit(f: &mut fmt::Formatter<'_>, id: &CommitId) -> fmt::Result {
    write!(
        f,
        "commit {} came from dam pull; the remote would send its changes again, so dam revert leaves it",
        id.short()
    )
}

pub(super) fn not_in_commit(
    f: &mut fmt::Formatter<'_>,
    commit: &CommitId,
    text: &str,
) -> fmt::Result {
    write!(f, "{text} is not in commit {}", commit.short())
}
