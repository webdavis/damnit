use super::run_push;
use crate::args::PushArgs;
use crate::context::Context;
use crate::testing::context;
use dam_application::{
    CredentialSpec, HelperError, HelperLauncher, RemoteConfig, RemoteHelper, RemoteName, Secret,
};

struct Uninstalled;
impl HelperLauncher for Uninstalled {
    fn launch(
        &self,
        remote: &RemoteConfig,
        _: &[(String, Secret)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        Err(HelperError::NotFound {
            helper: remote.helper.clone(),
        })
    }
}

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

#[test]
fn a_push_to_a_remote_whose_helper_is_not_installed_fails_as_a_helper_error() {
    let mut ctx = with_remote(vec![]);
    ctx.launcher = Box::new(Uninstalled);
    let error = run_push(&ctx, PushArgs { remote: None }).unwrap_err();
    assert_eq!(error.document()["error"]["kind"], "helper");
}

#[test]
fn a_push_whose_credential_resolves_to_nothing_fails_as_a_credential_error() {
    let ctx = with_remote(vec![CredentialSpec::Computed {
        remote: RemoteName("t".into()),
        name: "api_token".into(),
    }]);
    let error = run_push(&ctx, PushArgs { remote: None }).unwrap_err();
    assert_eq!(error.document()["error"]["kind"], "credential");
}
