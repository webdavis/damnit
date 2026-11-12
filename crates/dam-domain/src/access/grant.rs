use std::collections::BTreeSet;

use super::{KeyId, Permission};
use crate::{Object, Oid, Path};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Grant {
    pub allow: BTreeSet<Permission>,
    pub paths: Option<Vec<Path>>,
    pub labels: Option<BTreeSet<String>>,
    pub own: bool,
    pub remotes: Option<Vec<String>>,
}

#[derive(Clone, Copy, Debug)]
pub enum Subject<'a> {
    Object(&'a Object),
    Remote(&'a str),
    Whole,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grants {
    pub holder: KeyId,
    pub grants: Vec<Grant>,
    pub made: BTreeSet<Oid>,
}

impl Grants {
    pub fn holds(&self, permission: Permission) -> bool {
        self.grants.iter().any(|g| g.allow.contains(&permission))
    }

    pub fn permits(&self, permission: Permission, touched: &[Subject<'_>]) -> bool {
        !touched.is_empty()
            && self.grants.iter().any(|grant| {
                grant.allow.contains(&permission)
                    && touched
                        .iter()
                        .all(|s| grant.holds_for(permission, s, &self.made))
            })
    }
}

impl Grant {
    fn holds_for(
        &self,
        permission: Permission,
        subject: &Subject<'_>,
        made: &BTreeSet<Oid>,
    ) -> bool {
        match subject {
            Subject::Object(object) => {
                self.remotes.is_none()
                    && permission.kind().is_none_or(|kind| kind == object.kind())
                    && self.within_paths(&object.base().path)
                    && self.carries_a_label(&object.base().labels)
                    && (!self.own
                        || permission == Permission::create(object.kind())
                        || made.contains(object.oid()))
            }
            Subject::Remote(name) => {
                self.only_restricts_remotes()
                    && self
                        .remotes
                        .as_ref()
                        .is_none_or(|remotes| remotes.iter().any(|r| r == name))
            }
            Subject::Whole => self.only_restricts_remotes() && self.remotes.is_none(),
        }
    }

    fn only_restricts_remotes(&self) -> bool {
        self.paths.is_none() && self.labels.is_none() && !self.own
    }

    fn within_paths(&self, path: &Path) -> bool {
        self.paths
            .as_ref()
            .is_none_or(|prefixes| prefixes.iter().any(|p| path.is_within(p)))
    }

    fn carries_a_label(&self, labels: &BTreeSet<String>) -> bool {
        self.labels
            .as_ref()
            .is_none_or(|wanted| wanted.iter().any(|l| labels.contains(l)))
    }
}

#[cfg(test)]
mod tests;
