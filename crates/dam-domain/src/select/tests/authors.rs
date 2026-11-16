use super::fake::Fake;
use super::objects::{oid, task};
use super::tables::{one, table};
use crate::Object;
use crate::select::Filter;

fn hit(author: &str, object: &Object) -> bool {
    let mut fake = Fake::at("2026-11-20T12:00[UTC]");
    fake.authors.insert(oid(2), "scout".into());
    let filter = Filter::read(&table(&[("author", one(author))])).unwrap();
    filter.matches(object, &fake)
}

#[test]
fn author_matches_the_name_the_row_prints() {
    let (mine, agents) = (Object::Task(task(1)), Object::Task(task(2)));
    assert!(hit("owner", &mine) && !hit("owner", &agents));
    assert!(hit("scout", &agents) && !hit("scout", &mine));
    assert!(hit("owner,scout", &mine) && hit("owner,scout", &agents));
    assert!(hit("not:owner", &agents) && !hit("not:owner", &mine));
}
