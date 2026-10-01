mod accepted_kinds;
mod history;
mod reports;
mod unit_of_work;
mod working;

pub use accepted_kinds::accepted_kinds_repository_contract;
pub use history::{commit_repository_contract, remote_tracking_repository_contract};
pub use reports::{conflict_repository_contract, notice_repository_contract};
pub use unit_of_work::transactional_contract;
pub use working::{object_repository_contract, stage_repository_contract};

use dam_domain::{CommitId, Object, Oid, Path, Task};

use crate::ports::RemoteName;

fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

fn commit_id(byte: u8) -> CommitId {
    CommitId::generate(&mut |b: &mut [u8]| b.fill(byte))
}

fn task(byte: u8, subject: &str, path: &str) -> Object {
    let mut t = Task::new(oid(byte), subject);
    t.base.path = Path::parse(path).expect("the contract's own paths parse");
    Object::Task(t)
}

fn remote(name: &str) -> RemoteName {
    RemoteName(name.to_string())
}
