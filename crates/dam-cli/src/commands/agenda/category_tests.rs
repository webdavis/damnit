use dam_domain::{Categories, Category, Object};

use super::tests::standup;
use super::*;
use crate::testing::context;

#[test]
fn an_event_row_carries_the_label_of_each_group_it_holds() {
    let mut ctx = context();
    ctx.config.categories = Categories::new(vec![Category {
        name: "area".into(),
        values: vec!["work".into(), "home".into()],
        exclusive: true,
    }])
    .unwrap();
    ctx.store.put(&Object::Event(standup())).unwrap();
    let report = run_agenda(&ctx, AgendaArgs::default()).unwrap();
    assert_eq!(
        report.data["events"][0]["category"],
        serde_json::json!({"area": "work"})
    );
}
