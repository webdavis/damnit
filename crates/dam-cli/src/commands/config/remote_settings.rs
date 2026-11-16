use dam_application::{CredentialSpec, RemoteConfig, RemoteSync};

use super::setting::{Credential, Setting, Value};
use super::settings::{flag, named, optional, text};

pub(super) fn remote(remote: &RemoteConfig) -> (String, Setting) {
    let deadline = match &remote.deadline {
        Some(deadline) => Setting::set(Value::Text(deadline.text.clone())),
        None => Setting::by_default(Value::Text(RemoteConfig::DEFAULT_DEADLINE.to_string())),
    };
    let mut fields = vec![
        text("url", &remote.url),
        ("deadline".to_string(), deadline),
        sync(&remote.sync),
        optional("path", remote.path.as_ref().map(|path| path.as_str())),
        except(&remote.except),
    ];
    let credentials: Vec<_> = remote.credentials.iter().map(credential).collect();
    if !credentials.is_empty() {
        fields.push(named("credential", credentials));
    }
    named(&remote.name.0, fields)
}

fn except(paths: &[dam_domain::Path]) -> (String, Setting) {
    let setting = match paths {
        [] => Setting::unset(),
        _ => Setting::set(Value::List(
            paths.iter().map(|p| p.as_str().to_string()).collect(),
        )),
    };
    ("except".to_string(), setting)
}

fn sync(sync: &RemoteSync) -> (String, Setting) {
    let interval = match &sync.interval {
        Some(interval) => Setting::set(Value::Text(interval.text.clone())),
        None => Setting::by_default(Value::Text(RemoteSync::DEFAULT_INTERVAL.to_string())),
    };
    named(
        "sync",
        vec![
            ("interval".to_string(), interval),
            flag("allowed", sync.allowed.is_some(), sync.allowed()),
        ],
    )
}

fn credential(credential: &CredentialSpec) -> (String, Setting) {
    let hidden = match credential {
        CredentialSpec::Literal { .. } => Credential::Text,
        CredentialSpec::Computed { .. } => Credential::Function,
    };
    (credential.name().to_string(), Setting::Hidden(hidden))
}
