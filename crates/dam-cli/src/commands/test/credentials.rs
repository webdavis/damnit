use dam_adapters::LuaConfigFile;
use dam_application::{
    ConfigFile, CredentialSource, HelperError, HelperLauncher, LoadedConfig, Refusal, RemoteConfig,
    Secret, UseCaseError, resolve_credentials,
};

use crate::args::CheckName;
use crate::commands::check_report::{Check, Outcome, Row};
use crate::error::{ConfigReason, UseCaseText};

pub(super) fn test(config_file: &LuaConfigFile, launcher: &dyn HelperLauncher) -> Check {
    let found = |result, detail: String| Check::of(CheckName::Credentials, result, detail);
    let loaded = config_file.ends_in_lua().and_then(|()| config_file.load());
    let LoadedConfig {
        config,
        credentials,
        ..
    } = match loaded {
        Ok(loaded) => loaded,
        Err(error) => {
            let detail = format!(
                "not tested, the config did not read: {}",
                ConfigReason(&error)
            );
            return found(Outcome::Issue, detail).advising("run dam doctor config");
        }
    };
    if config.remotes.is_empty() {
        return found(Outcome::NotRun, "no remotes configured".into());
    }
    let rows = config
        .remotes
        .iter()
        .map(|remote| signed_in(remote, credentials.as_ref(), launcher))
        .collect();
    Check::of_rows(CheckName::Credentials, rows)
}

fn signed_in(
    remote: &RemoteConfig,
    source: &dyn CredentialSource,
    launcher: &dyn HelperLauncher,
) -> Row {
    let helper = format!("dam-remote-{}", remote.helper);
    let found = |result, detail: String| Row::of(&remote.name.0, result, detail);
    let asked = launcher
        .launch(remote, &[])
        .and_then(|mut h| h.capabilities());
    let capabilities = match asked {
        Ok(capabilities) => capabilities,
        Err(HelperError::NotFound { .. }) => {
            return found(Outcome::Issue, format!("{helper} is not installed"))
                .advising("install it, then run dam test credentials");
        }
        Err(error) => return found(Outcome::Issue, helper_text(error)),
    };
    if !capabilities.answers_login {
        return found(Outcome::Warning, format!("{helper} cannot test a login"))
            .advising(format!("update {helper}"));
    }
    let resolved = match resolve_credentials(source, remote, &capabilities.credentials) {
        Ok(resolved) => resolved,
        Err(UseCaseError::Refused(Refusal::MissingCredential { name, .. })) => {
            let remote = &remote.name.0;
            return found(Outcome::Issue, format!("has no {name}"))
                .advising(format!("set {name} under remote.{remote} in secrets.lua"));
        }
        Err(error) => return found(Outcome::Issue, UseCaseText(&error).to_string()),
    };
    match launcher
        .launch(remote, &resolved)
        .and_then(|mut h| h.login())
    {
        Ok(()) => found(Outcome::Ok, "signed in".into()),
        Err(HelperError::Remote(said)) => {
            let detail = format!("did not sign in: {}", withheld(&said, &resolved));
            found(Outcome::Issue, detail).advising("check its credentials in secrets.lua")
        }
        Err(error) => found(Outcome::Issue, withheld(&helper_text(error), &resolved)),
    }
}

fn helper_text(error: HelperError) -> String {
    UseCaseText(&UseCaseError::Helper(error)).to_string()
}

fn withheld(text: &str, credentials: &[(String, Secret)]) -> String {
    credentials
        .iter()
        .filter(|(_, value)| !value.expose().is_empty())
        .fold(text.to_string(), |text, (name, value)| {
            text.replace(value.expose(), &format!("<{name}>"))
        })
}
