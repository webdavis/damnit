use super::*;
use crate::args::{RemoteArgs, RemoteCommand};
use crate::commands::remote::run_remote;
use crate::testing::context;
use dam_application::RemoteConfig;
use dam_application::RemoteName;

#[test]
fn remote_list_reports_the_last_pull_and_push_as_rfc_3339_a_phrase_or_null() {
    let mut ctx = context();
    let t = RemoteName("t".into());
    ctx.config.remotes.push(RemoteConfig {
        name: t.clone(),
        helper: "t".into(),
        url: "t::".into(),
        credentials: vec![],
        deadline: None,
        sync: Default::default(),
        path: None,
        except: Vec::new(),
    });
    let never = listed(&ctx);
    assert_eq!(
        never.data["remotes"][0]["last_pull"],
        serde_json::Value::Null
    );
    assert_eq!(
        never.data["remotes"][0]["last_push"],
        serde_json::Value::Null
    );
    assert!(never.human.contains("never pulled"), "{}", never.human);
    assert!(never.human.contains("never pushed"), "{}", never.human);

    let now = ctx.clock.now();
    ctx.store
        .set_last_pull(&t, now - std::time::Duration::from_secs(240))
        .unwrap();
    ctx.store
        .set_last_push(&t, now - std::time::Duration::from_secs(7200))
        .unwrap();
    let report = listed(&ctx);
    assert_eq!(
        report.data["remotes"][0]["last_pull"],
        (now - std::time::Duration::from_secs(240)).to_string()
    );
    assert_eq!(
        report.data["remotes"][0]["last_push"],
        (now - std::time::Duration::from_secs(7200)).to_string()
    );
    assert!(report.human.contains("pulled 4m ago"), "{}", report.human);
    assert!(report.human.contains("pushed 2h ago"), "{}", report.human);
}

fn listed(ctx: &Context) -> Report {
    run_remote(
        ctx,
        RemoteArgs {
            command: RemoteCommand::List,
        },
    )
    .unwrap()
}

#[test]
fn no_remotes_is_named_in_one_sentence() {
    assert_eq!(listed(&context()).human, "no remotes configured");
}

#[test]
fn a_clock_gone_backwards_reads_as_just_now_rather_than_the_future() {
    let now = context().clock.now();
    let later = now + std::time::Duration::from_secs(7200);
    assert_eq!(
        ago_to_one_unit_and_never_in_the_future(now, later),
        "just now"
    );
}

#[test]
fn remote_list_shows_the_paths_a_remote_never_receives_beside_its_path() {
    let mut ctx = context();
    ctx.config.remotes.push(RemoteConfig {
        name: RemoteName("t".into()),
        helper: "t".into(),
        url: "t::".into(),
        credentials: vec![],
        deadline: None,
        sync: Default::default(),
        path: Some(dam_domain::Path::parse("home").unwrap()),
        except: vec![
            dam_domain::Path::parse("home/kitchen").unwrap(),
            dam_domain::Path::parse("home/journal").unwrap(),
        ],
    });
    let report = listed(&ctx);
    assert_eq!(
        report.data["remotes"][0]["except"],
        serde_json::json!(["home/kitchen/", "home/journal/"])
    );
    assert!(
        report
            .human
            .contains("path home/  except home/kitchen/, home/journal/"),
        "{}",
        report.human
    );
}
