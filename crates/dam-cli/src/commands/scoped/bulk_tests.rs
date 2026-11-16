mod edit_categories;
mod flags;
mod tokens;

use clap::Parser;
use dam_application::testing::task_in;
use dam_application::{ConfigError, FilterConfig, FilterRule, Project, ProjectSettings};
use dam_domain::{Categories, Category, Object};
use serde_json::Value;

use super::{dispatch_in, following};
use crate::args::Cli;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::project_scope::Standing;
use crate::testing::context;

const VERBS: [&[&str]; 4] = [
    &["done"],
    &["edit", "--subject", "x"],
    &["mv", "--path", "moved/"],
    &["rm"],
];

fn settings() -> ProjectSettings {
    ProjectSettings {
        path: "damnit/".into(),
        ..Default::default()
    }
}

fn inside_with(settings: ProjectSettings) -> Standing {
    Standing::Inside(Box::new(Project {
        folder: "/work/damnit".into(),
        settings,
    }))
}

fn inside() -> Standing {
    inside_with(settings())
}

fn unreadable() -> Standing {
    Standing::Unreadable(ConfigError::NotATable {
        path: "/work/damnit/.dam.lua".into(),
        returned: "a number".into(),
    })
}

fn tasks_filter(query: &str) -> FilterConfig {
    FilterConfig {
        name: "tasks".into(),
        rule: FilterRule::Query(query.into()),
    }
}

fn stage() -> Categories {
    Categories::new(vec![Category {
        name: "stage".into(),
        values: vec!["review".into(), "ship".into()],
        exclusive: true,
    }])
    .unwrap()
}

fn holding_a_project_and_more() -> Context {
    let mut ctx = context();
    ctx.config.filters = vec![tasks_filter("kind:task")];
    for task in [
        task_in(1, "readme", "damnit"),
        task_in(2, "parse flags", "damnit/cli"),
        task_in(3, "other cli", "cli"),
        task_in(4, "laundry", "home"),
    ] {
        ctx.store.put(&Object::Task(task)).unwrap();
    }
    ctx
}

fn oid_of(ctx: &Context, subject: &str) -> String {
    let all = ctx.store.all().unwrap();
    let found = all.iter().find(|o| o.base().subject == subject).unwrap();
    found.oid().as_str().into()
}

fn run(ctx: Context, words: &[&str], standing: Standing) -> Result<Report, CliError> {
    let line = std::iter::once("dam").chain(words.iter().copied());
    let command = Cli::try_parse_from(line)
        .map_err(|e| CliError::Usage(e.to_string()))?
        .command;
    let ctx = following(ctx, &command, &standing);
    dispatch_in(&ctx, command, standing)
}

fn picked(ctx: Context, verb: &[&str], rest: &[&str], standing: Standing) -> Vec<String> {
    let subjects: Vec<(String, String)> = ctx
        .store
        .all()
        .unwrap()
        .iter()
        .map(|o| (o.oid().as_str().to_string(), o.base().subject.clone()))
        .collect();
    let words: Vec<&str> = [verb, &["--preview"], rest].concat();
    let report = run(ctx, &words, standing).unwrap();
    let mut picked: Vec<String> = report.data["oids"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .map(|oid| {
            let (_, subject) = subjects.iter().find(|(o, _)| o == oid).unwrap();
            subject.clone()
        })
        .collect();
    picked.sort();
    picked
}

fn each_verb_picks(rest: &[&str], standing: impl Fn() -> Standing, expected: &[&str]) {
    for verb in VERBS {
        let picked = picked(holding_a_project_and_more(), verb, rest, standing());
        assert_eq!(picked, expected, "dam {verb:?} {rest:?}");
    }
}
