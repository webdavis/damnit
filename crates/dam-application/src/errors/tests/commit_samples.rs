use crate::errors::Refusal;
use dam_domain::CommitId;

pub(super) fn commit_samples() -> Vec<Refusal> {
    let commit_id = || CommitId::generate(&mut |b: &mut [u8]| b.fill(9));
    vec![
        Refusal::PulledCommit(commit_id()),
        Refusal::NotInCommit {
            commit: commit_id(),
            text: "abab".into(),
        },
    ]
}
