use clap::Parser;
use dam_application::{CredentialSpec, RemoteConfig, RemoteName, RunEnding};
use jiff::civil::date;

use super::run_sync;
use crate::args::{Cli, Command};
use crate::context::Context;
use crate::testing::context;

fn with_remote(credentials: Vec<CredentialSpec>) -> Context {
    let mut ctx = context();
    ctx.config.remotes.push(RemoteConfig {
        name: RemoteName("t".into()),
        helper: "t".into(),
        url: "t::".into(),
        credentials,
        deadline: None,
        sync: Default::default(),
        path: None,
        except: Vec::new(),
    });
    ctx
}

fn sync(ctx: &Context, line: &str) {
    let words = std::iter::once("dam").chain(line.split(' '));
    let Command::Sync(args) = Cli::try_parse_from(words).unwrap().command else {
        panic!("not sync")
    };
    let _ = run_sync(ctx, args);
}

fn ended(ctx: &Context) -> RunEnding {
    let last = ctx.run_records.last("t").unwrap().unwrap();
    let midnight = date(2026, 9, 18).to_zoned(jiff::tz::TimeZone::UTC).unwrap();
    assert_eq!(last.at, midnight.timestamp());
    last.ended
}

#[test]
fn a_run_when_due_records_a_sync_and_then_that_the_next_was_not_due() {
    let ctx = with_remote(vec![]);
    sync(&ctx, "sync run t --when-due");
    assert_eq!(ended(&ctx), RunEnding::Synced);
    sync(&ctx, "sync run t --when-due");
    assert_eq!(ended(&ctx), RunEnding::NotDue);
}

#[test]
fn a_failed_run_records_one_line_of_why() {
    let missing = CredentialSpec::Computed {
        remote: RemoteName("t".into()),
        name: "token".into(),
    };
    let ctx = with_remote(vec![missing]);
    sync(&ctx, "sync run t --when-due");
    let RunEnding::Failed(why) = ended(&ctx) else {
        panic!("not failed")
    };
    assert!(why.contains("token") && !why.contains('\n'), "{why}");
}

#[test]
fn a_run_by_hand_is_recorded_too() {
    let ctx = with_remote(vec![]);
    sync(&ctx, "sync run t");
    assert_eq!(ended(&ctx), RunEnding::Synced);
}

struct RaisingSecrets;

impl dam_application::CredentialSource for RaisingSecrets {
    fn resolve(
        &self,
        spec: &CredentialSpec,
    ) -> Result<dam_application::Secret, dam_application::CredentialError> {
        Err(dam_application::CredentialError::ComputeFailed {
            remote: RemoteName("t".into()),
            name: spec.name().into(),
            at: "secrets.lua:3".into(),
            why: dam_application::ComputeFailure::Raised {
                message: Some("hunter2 was refused".into()),
            },
        })
    }
}

#[test]
fn a_failed_credential_function_is_recorded_without_its_message() {
    let computed = CredentialSpec::Computed {
        remote: RemoteName("t".into()),
        name: "token".into(),
    };
    let mut ctx = with_remote(vec![computed]);
    ctx.credentials = Box::new(RaisingSecrets);
    sync(&ctx, "sync run t --when-due");
    let RunEnding::Failed(why) = ended(&ctx) else {
        panic!("not failed")
    };
    assert!(!why.contains("hunter2"), "{why}");
    assert!(why.contains("dam sync run t"), "{why}");
}
