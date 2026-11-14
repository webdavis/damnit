use dam_adapters::history_document;
use dam_domain::{
    Author, Authored, Change, Channel, CommitId, CommitRecord, Details, Event, Object, Oid,
    Permission, Task, Timestamp, Token, TokenAuthor, When,
};
use jiff::civil::date;

use crate::context::Context;
use crate::testing::{context, holding};

fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

fn by_a_token(name: &str, change: Change) -> Authored {
    Authored {
        change,
        author: Author::Token(TokenAuthor {
            name: name.into(),
            key_id: Token::generate(&mut |b: &mut [u8]| b.fill(1))
                .key_id()
                .clone(),
            channel: Channel::Local,
        }),
        details: Some(Details {
            meta: [("session".to_string(), "s1".to_string())].into(),
            host: Some("host".into()),
            dir: None,
            repo: None,
            branch: None,
            at: Timestamp::UNIX_EPOCH,
            channel: Channel::Local,
        }),
    }
}

fn a_plan(ctx: &Context) -> CommitRecord {
    let mut task = Task::new(oid(1), "report");
    task.due = Some(When::Day(date(2030, 1, 1)));
    let mut later = task.clone();
    later.due = Some(When::Day(date(2030, 1, 2)));
    let event = Event::new(
        oid(2),
        "standup",
        When::Day(date(2030, 1, 1)),
        When::Day(date(2030, 1, 2)),
    );
    let record = CommitRecord {
        id: CommitId::generate(&mut |b: &mut [u8]| b.fill(9)),
        message: "plan".into(),
        at: Timestamp::UNIX_EPOCH,
        changes: vec![
            Authored::by_owner(Change::Update {
                before: Object::Task(task.clone()),
                after: Object::Task(later),
            }),
            by_a_token(
                "tool",
                Change::Create {
                    after: Object::Event(event),
                },
            ),
            by_a_token(
                "work",
                Change::Delete {
                    before: Object::Task(task),
                },
            ),
        ],
    };
    ctx.store.commit(&record).unwrap();
    record
}

#[test]
fn the_history_a_filter_function_reads_is_what_dam_log_prints() {
    let ctx = context();
    let record = a_plan(&ctx);
    let log = super::run_log(&ctx).unwrap().data;
    assert_eq!(
        history_document(&[record], ctx.config.owner_name()),
        log["commits"]
    );
}

#[test]
fn a_tokens_narrowed_history_is_what_dam_log_prints_to_that_token() {
    let ctx = Context {
        access: holding(&[Permission::TaskRead, Permission::HistoryRead]),
        ..context()
    };
    let mut narrowed = a_plan(&ctx);
    narrowed.changes[2].details = None;
    let log = super::run_log(&ctx).unwrap().data;
    assert_eq!(
        log["commits"][0]["changes"][2]["details"],
        serde_json::Value::Null
    );
    assert_eq!(
        history_document(&[narrowed], ctx.config.owner_name()),
        log["commits"]
    );
}
