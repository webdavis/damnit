use dam_application::{RemoteConfig, RemoteName};
use dam_domain::Permission::RemoteRead;

use super::*;
use crate::args::{RemoteArgs, RemoteCommand};
use crate::testing::{context, holding};

fn remote(name: &str) -> RemoteConfig {
    RemoteConfig {
        name: RemoteName(name.into()),
        helper: name.into(),
        url: format!("{name}::"),
        credentials: vec![],
        stale: None,
        deadline: None,
        path: None,
    }
}

#[test]
fn a_token_lists_only_the_remotes_its_remote_read_names() {
    let mut ctx = context();
    ctx.config.remotes = vec![remote("todoist"), remote("gcal")];
    let mut token = holding(&[RemoteRead]);
    token.grants.grants[0].remotes = Some(vec!["gcal".into()]);
    ctx.access = token;
    let listed = run_remote(
        &ctx,
        RemoteArgs {
            command: RemoteCommand::List,
        },
    )
    .unwrap();
    let names: Vec<&str> = listed.data["remotes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["gcal"]);
}
