use std::collections::BTreeSet;

use dam_domain::Oid;

use crate::config::RemoteConfig;

#[derive(Default)]
pub(super) struct Hint {
    pub(super) excepted: BTreeSet<Oid>,
    remote: String,
    scoped: bool,
}

impl Hint {
    pub(super) fn for_remote(remote: &RemoteConfig) -> Hint {
        Hint {
            excepted: BTreeSet::new(),
            remote: remote.name.0.clone(),
            scoped: remote.path.is_some(),
        }
    }

    pub(super) fn place_for(&self, stuck: &Oid) -> String {
        if self.excepted.contains(stuck) {
            format!("PATH outside remote.{}.except", self.remote)
        } else if self.scoped {
            "PATH inside this remote's path".to_string()
        } else {
            "PATH".to_string()
        }
    }
}
