use dam_application::ProjectSettings;
use dam_domain::{Event, Object, Oid, Path, When};

use super::super::dispatch_in;
use super::inside;
use crate::args::Command;
use crate::context::Context;
use crate::project_scope::Standing;
use crate::testing::{agenda_args, context};

fn meeting(fill: u8, subject: &str, label: Option<&str>) -> Object {
    let mut event = Event::new(
        Oid::generate(&mut |b: &mut [u8]| b.fill(fill)),
        subject,
        When::At("2026-09-18T14:00:00+00:00[UTC]".parse().unwrap()),
        When::At("2026-09-18T14:30:00+00:00[UTC]".parse().unwrap()),
    );
    event.base.path = Path::parse("app").unwrap();
    event.base.labels.extend(label.map(String::from));
    Object::Event(event)
}

fn holding() -> Context {
    let ctx = context();
    for event in [
        meeting(1, "standup", Some("work")),
        meeting(2, "lunch", None),
    ] {
        ctx.store.put(&event).unwrap();
    }
    ctx
}

fn with_a_work_filter() -> Standing {
    inside(ProjectSettings {
        filters: vec![dam_application::testing::saved(
            "work",
            &[("kind", "any"), ("labels", "work")],
        )],
        ..ProjectSettings::default()
    })
}

fn agenda(filter: &str, global: bool) -> Command {
    let mut args = agenda_args(&["--filter", filter]);
    args.scope.global = global;
    Command::Agenda(args)
}

fn subjects(ctx: &Context, command: Command, standing: Standing) -> Vec<String> {
    let listed = dispatch_in(ctx, command, standing).unwrap();
    let events = listed.data["events"].as_array().unwrap();
    events
        .iter()
        .map(|e| e["subject"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn agenda_inside_the_project_reads_a_project_filter_by_name() {
    let ctx = holding();
    assert_eq!(
        subjects(&ctx, agenda("work", false), with_a_work_filter()),
        ["standup"]
    );
}

#[test]
fn agenda_with_global_does_not_know_the_projects_filter() {
    let ctx = holding();
    assert!(dispatch_in(&ctx, agenda("work", true), with_a_work_filter()).is_err());
}
