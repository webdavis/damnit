use dam_domain::{Change, Channel, Item, Object, Oid, Path, Task, Timestamp, Token, TokenAuthor};

use super::ItemRow;
use crate::view::Line;

fn only(lines: Vec<Line>) -> String {
    match lines.as_slice() {
        [Line::Row(line)] => line.clone(),
        _ => panic!("a waiting new item reads as one row"),
    }
}

fn dentist_sent_at(created: &str) -> Item {
    let mut task = Task::new(
        Oid::generate(&mut |b: &mut [u8]| b.fill(0xa1)),
        "Call the dentist",
    );
    task.base.path = Path::parse("inbox").unwrap();
    Item::propose(
        TokenAuthor {
            name: "tool".into(),
            key_id: Token::generate(&mut |b: &mut [u8]| b.fill(1))
                .key_id()
                .clone(),
            channel: Channel::Local,
        },
        Change::Create {
            after: Object::Task(task),
        },
        created.parse().unwrap(),
        &mut |b| b.fill(2),
    )
}

#[test]
fn a_waiting_item_reads_as_what_it_does_its_hash_path_sender_and_age() {
    let item = dentist_sent_at("2026-11-11T07:00:00Z");
    let now: Timestamp = "2026-11-11T09:30:00Z".parse().unwrap();
    let row = ItemRow::new(&item, now, false);
    assert_eq!(
        only(row.lines("you")),
        "  new      a1a1a1a  Call the dentist  inbox/  tool  2h"
    );
    let json = serde_json::to_value(&row).unwrap();
    assert_eq!(json["hash"], item.hash.to_string());
    assert_eq!(json["item"], "new");
    assert_eq!(json["oid"], item.hash.to_string());
    assert_eq!(json["from"], "tool");
    assert_eq!(json["channel"], "local");
    assert_eq!(json["created"], "2026-11-11T07:00:00Z");
    assert_eq!(json["age"], 9000);
}

#[test]
fn age_reads_in_the_largest_whole_unit() {
    let item = dentist_sent_at("2026-11-11T09:00:00Z");
    let at = |now: &str| only(ItemRow::new(&item, now.parse().unwrap(), false).lines("you"));
    assert!(at("2026-11-11T09:00:45Z").ends_with("tool  45s"));
    assert!(at("2026-11-11T09:05:00Z").ends_with("tool  5m"));
    assert!(at("2026-11-14T09:00:00Z").ends_with("tool  3d"));
}
