use crate::config::{Config, CredentialSpec, RemoteConfig};
use crate::ports::RemoteName;

pub fn todoist() -> RemoteName {
    RemoteName("todoist".into())
}

pub fn config() -> Config {
    Config {
        remotes: vec![RemoteConfig {
            name: todoist(),
            helper: "todoist".into(),
            url: "todoist::".into(),
            credentials: vec![CredentialSpec::Literal {
                name: "api_token".into(),
                value: "t".into(),
            }],
            deadline: None,
            sync: Default::default(),
            path: None,
        }],
        ..Config::default()
    }
}
