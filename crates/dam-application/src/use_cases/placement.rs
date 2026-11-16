use dam_domain::{Object, Oid, Path, Permission, nearest_parents};

use crate::errors::{Refusal, Retry, UseCaseError};
use crate::service::Dam;
use crate::use_cases::load::no_such_object;

pub(crate) struct Placement<'a> {
    pub(crate) path: Option<&'a Path>,
    pub(crate) parent: Option<&'a Oid>,
    pub(crate) writing: Option<&'a Object>,
}

impl Dam<'_> {
    pub(crate) fn placement(
        &self,
        asked: Placement<'_>,
        retry: impl FnOnce() -> Retry,
    ) -> Result<(Path, Option<Oid>), UseCaseError> {
        let path = asked.path.filter(|p| **p != Path::root());
        let Some(parent) = asked.parent else {
            let path = path.cloned().unwrap_or_else(Path::root);
            self.refuse_unless_writable(asked.writing, &path)?;
            self.refuse_several_holding(asked.writing, &path, retry)?;
            return Ok((path, None));
        };
        if let Some(path) = path {
            self.refuse_unless_writable(asked.writing, path)?;
        }
        let folder = self.folder_of(parent)?;
        let path = match path {
            Some(path) => path.clone(),
            None => {
                self.refuse_unless_writable(asked.writing, &folder)?;
                folder.clone()
            }
        };
        if !path.is_within(&folder) {
            return Err(Refusal::PathOutsideParent {
                parent: parent.clone(),
                folder,
                path,
            }
            .into());
        }
        Ok((path, Some(parent.clone())))
    }

    fn folder_of(&self, parent: &Oid) -> Result<Path, UseCaseError> {
        let object = self
            .store
            .get(parent)?
            .filter(|o| self.access.sees(o))
            .ok_or_else(|| no_such_object(parent))?;
        Ok(object
            .base()
            .owned_folder()
            .ok_or_else(|| Refusal::SubjectNamesNoFolder(parent.clone()))?)
    }

    fn refuse_unless_writable(
        &self,
        writing: Option<&Object>,
        path: &Path,
    ) -> Result<(), UseCaseError> {
        let Some(object) = writing else {
            return Ok(());
        };
        let mut placed = object.clone();
        placed.base_mut().path = path.clone();
        self.refuse_unless_permitted(Permission::create(object.kind()), None, &placed)
    }

    fn refuse_several_holding(
        &self,
        writing: Option<&Object>,
        path: &Path,
        retry: impl FnOnce() -> Retry,
    ) -> Result<(), UseCaseError> {
        let all = self.store.all()?;
        let holding = nearest_parents(path, &all);
        if holding.len() < 2 {
            return Ok(());
        }
        let seen = all
            .iter()
            .filter(|o| holding.contains(o.oid()) && self.access.sees(o))
            .count();
        if let Some(object) = writing
            && seen < holding.len()
        {
            return Err(Refusal::NotPermitted(Permission::create(object.kind())).into());
        }
        Err(Refusal::SeveralHoldThePath {
            namesakes: holding,
            folder: path.clone(),
            retry: retry(),
        }
        .into())
    }
}
