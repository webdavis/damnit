use dam_domain::{Categories, Category, Event, When};
use serde_json::{Value, json};

use super::*;

fn first_change(report: &Report) -> &Value {
    &report.data["changes"][0]
}

fn with_effort(mut ctx: Context) -> Context {
    ctx.config.categories = Categories::new(vec![Category {
        name: "effort".into(),
        values: vec!["quick".into(), "deep".into()],
        exclusive: true,
    }])
    .unwrap();
    ctx
}

#[test]
fn a_preview_carries_the_deadline_estimate_and_category_it_would_write() {
    let (ctx, oids) = three_tasks();
    let ctx = with_effort(ctx);
    let out = dam(
        &ctx,
        &[
            "edit",
            &oids[0],
            "--deadline",
            "2026-12-04",
            "--estimate",
            "30",
            "--category",
            "effort=quick",
            "--preview",
        ],
    )
    .unwrap();
    let change = first_change(&out);
    assert_eq!(change["deadline"], "2026-12-04", "{change}");
    assert_eq!(change["estimate"], 30, "{change}");
    assert_eq!(change["category"], json!({"effort": "quick"}), "{change}");
}

#[test]
fn a_write_on_several_says_what_its_preview_says() {
    let (ctx, oids) = three_tasks();
    let words = [
        "edit",
        &oids[0],
        &oids[1],
        "--estimate",
        "45",
        "--body",
        "notes",
    ];
    let preview = dam(&ctx, &[&words[..], &["--preview"]].concat()).unwrap();
    let written = dam(&ctx, &words).unwrap();
    for out in [preview, written] {
        assert_eq!(first_change(&out)["estimate"], 45, "{}", out.data);
        assert_eq!(first_change(&out)["body"], "notes", "{}", out.data);
    }
}

#[test]
fn a_cleared_field_carries_null() {
    let (ctx, oids) = three_tasks();
    dam(&ctx, &["edit", &oids[0], &oids[1], "--estimate", "45"]).unwrap();
    let out = dam(&ctx, &["edit", &oids[0], "--no-estimate", "--preview"]).unwrap();
    let change = first_change(&out);
    assert_eq!(change["fields"], json!(["estimate"]), "{change}");
    assert_eq!(change["estimate"], Value::Null, "{change}");
    assert!(change.as_object().unwrap().contains_key("estimate"));
}

#[test]
fn a_field_that_does_not_change_is_left_out() {
    let (ctx, oids) = three_tasks();
    let out = dam(&ctx, &["edit", &oids[0], "--estimate", "5", "--preview"]).unwrap();
    let change = first_change(&out).as_object().unwrap();
    assert!(!change.contains_key("deadline"));
    assert!(!change.contains_key("category"));
}

#[test]
fn a_move_carries_the_parent_it_would_have() {
    let (ctx, oids) = three_tasks();
    let out = dam(&ctx, &["mv", &oids[0], "garden/", "--preview"]).unwrap();
    let change = first_change(&out);
    assert_eq!(change["path"], "garden/", "{change}");
    assert_eq!(change["parent"], Value::Null, "{change}");
    assert!(change.as_object().unwrap().contains_key("parent"));
}

#[test]
fn an_event_preview_carries_its_location() {
    let ctx = context();
    let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(9));
    let day = When::Day(jiff::civil::date(2026, 12, 1));
    ctx.store
        .put(&Object::Event(Event::new(
            oid.clone(),
            "meet",
            day.clone(),
            day,
        )))
        .unwrap();
    let out = dam(
        &ctx,
        &["edit", oid.as_str(), "--location", "hall", "--preview"],
    )
    .unwrap();
    assert_eq!(first_change(&out)["location"], "hall", "{}", out.data);
}
