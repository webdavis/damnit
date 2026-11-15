use dam_application::Refusal;
use dam_domain::CommitId;

fn commit_id() -> CommitId {
    CommitId::generate(&mut |b: &mut [u8]| b.fill(9))
}

pub(super) fn history_refusals_with_their_text() -> Vec<(Refusal, &'static str)> {
    vec![
        (
            Refusal::PulledCommit(commit_id()),
            "commit 0909090 came from dam pull; the remote would send its changes again, so dam revert leaves it",
        ),
        (
            Refusal::NotInCommit {
                commit: commit_id(),
                text: "abab".into(),
            },
            "abab is not in commit 0909090",
        ),
    ]
}
