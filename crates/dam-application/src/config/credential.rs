use crate::ports::RemoteName;
use crate::secret::Secret;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CredentialSpec {
    Literal { name: String, value: Secret },
    Command { name: String, argv: Vec<String> },
    Env { name: String, var: String },
    Computed { remote: RemoteName, name: String },
}

impl CredentialSpec {
    pub fn name(&self) -> &str {
        match self {
            CredentialSpec::Literal { name, .. }
            | CredentialSpec::Command { name, .. }
            | CredentialSpec::Env { name, .. }
            | CredentialSpec::Computed { name, .. } => name,
        }
    }
}
