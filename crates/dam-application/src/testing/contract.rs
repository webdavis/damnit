mod accepted_kinds;
mod days;
mod history;
mod ids;
mod items;
mod remote_tracking;
mod reports;
mod unit_of_work;
mod working;

pub use accepted_kinds::accepted_kinds_repository_contract;
pub use days::day_repository_contract;
pub use history::{commit_clears_the_stage_contract, commit_repository_contract};
pub use ids::id_repository_contract;
pub use items::item_repository_contract;
pub use remote_tracking::remote_tracking_repository_contract;
pub use reports::{conflict_repository_contract, notice_repository_contract};
pub use unit_of_work::transactional_contract;
pub use working::{object_repository_contract, stage_repository_contract};

use dam_domain::{Author, Channel, CommitId, Object, Path, Task, Token, TokenAuthor};

use crate::ports::RemoteName;
use crate::testing::oid;

fn commit_id(byte: u8) -> CommitId {
    CommitId::generate(&mut |b: &mut [u8]| b.fill(byte))
}

fn task(byte: u8, subject: &str, path: &str) -> Object {
    let mut t = Task::new(oid(byte), subject);
    t.base.path = Path::parse(path).expect("the contract's own paths parse");
    Object::Task(t)
}

fn token_author() -> Author {
    Author::Token(TokenAuthor {
        name: "tool".into(),
        key_id: Token::generate(&mut |b: &mut [u8]| b.fill(3))
            .key_id()
            .clone(),
        channel: Channel::Local,
    })
}

fn remote(name: &str) -> RemoteName {
    RemoteName(name.to_string())
}
