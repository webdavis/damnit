use std::collections::BTreeSet;

use dam_adapters::SqliteStore;
use dam_application::ObjectRepository;
use dam_domain::{Event, Object, Oid, Path, Priority, Task, When};

use super::sandbox::Sandbox;

pub const OAT_MILK: u8 = 0x11;
pub const CALL_MOM: u8 = 0x22;
pub const STANDUP: u8 = 0x33;
pub const OFFSITE: u8 = 0x44;

pub fn oid(byte: u8) -> String {
    byte_oid(byte).as_str().to_string()
}

pub fn short(byte: u8) -> String {
    oid(byte)[..7].to_string()
}

pub fn seeded_oids() -> Vec<String> {
    [OAT_MILK, CALL_MOM, STANDUP, OFFSITE].map(oid).to_vec()
}

pub fn seeded() -> Sandbox {
    let sb = Sandbox::new();
    let config = sb.dir.path().join("config.toml");
    let mut text = std::fs::read_to_string(&config).unwrap();
    text.push_str(
        "\n[category.effort]\nvalues = [\"light\", \"deep\"]\nexclusive = true\n\
         \n[category.context]\nvalues = [\"home\", \"errand\"]\n\
         \n[filter.today]\nquery = \"due:today | overdue\"\n",
    );
    std::fs::write(&config, text).unwrap();
    let store = SqliteStore::open(&sb.dir.path().join("dam.db")).unwrap();
    for object in mixed_rows() {
        store.put(&object).unwrap();
    }
    sb
}

pub fn committed() -> Sandbox {
    let sb = seeded();
    run(&sb, &["add", "-A"]);
    run(&sb, &["commit", "-m", "seed the week"]);
    sb
}

pub fn run(sb: &Sandbox, args: &[&str]) {
    let (ok, _, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
}

fn mixed_rows() -> Vec<Object> {
    let mut oat_milk = Task::new(byte_oid(OAT_MILK), "buy oat milk");
    oat_milk.priority = Priority::new(2).unwrap();
    oat_milk.due = Some(When::Day("2026-09-25".parse().unwrap()));
    oat_milk.base.labels = BTreeSet::from(["errand".to_string()]);
    oat_milk.base.path = Path::parse("home").unwrap();
    let call_mom = Task::new(byte_oid(CALL_MOM), "call mom");
    let mut standup = Event::new(
        byte_oid(STANDUP),
        "standup",
        utc("2026-09-25T09:00:00+00:00[UTC]"),
        utc("2026-09-25T09:15:00+00:00[UTC]"),
    );
    standup.base.path = Path::parse("work").unwrap();
    let mut offsite = Event::new(
        byte_oid(OFFSITE),
        "offsite",
        When::Day("2026-09-25".parse().unwrap()),
        When::Day("2026-09-27".parse().unwrap()),
    );
    offsite.base.labels = BTreeSet::from(["deep".to_string()]);
    offsite.base.path = Path::parse("work").unwrap();
    vec![
        Object::Task(oat_milk),
        Object::Task(call_mom),
        Object::Event(standup),
        Object::Event(offsite),
    ]
}

fn utc(text: &str) -> When {
    When::At(text.parse().unwrap())
}

fn byte_oid(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

pub fn with_staged_and_unstaged_changes() -> Sandbox {
    let sb = committed();
    run(
        &sb,
        &["edit", &short(OAT_MILK), "--subject", "buy soy milk"],
    );
    run(&sb, &["rm", &short(CALL_MOM)]);
    run(&sb, &["add", &oid(CALL_MOM)]);
    sb
}

pub fn with_two_commits() -> Sandbox {
    let sb = committed();
    run(&sb, &["done", &short(OAT_MILK)]);
    run(&sb, &["add", "-A"]);
    run(&sb, &["commit", "-m", "close the errand"]);
    sb
}

pub fn pulled_and_pushed() -> Sandbox {
    let sb = committed();
    run(&sb, &["pull"]);
    run(&sb, &["push"]);
    sb
}
