use dam_domain::{Author, CallerId, IdHolder, IdKey, ItemHash, Object, Oid, Permission};

use crate::errors::UseCaseError;
use crate::service::Dam;
use crate::use_cases::items::land::Landed;

impl Dam<'_> {
    pub(crate) fn land_new(
        &self,
        id: Option<&CallerId>,
        fresh: Object,
    ) -> Result<Landed, UseCaseError> {
        let create = Permission::create(fresh.kind());
        let Some(id) = id else {
            self.refuse_unpermitted_or_unlabelled(create, None, &fresh)?;
            return self.put_or_send(create, None, fresh);
        };
        let key = self.id_key(id);
        let mapped = self.store.oid_for_id(&key)?;
        if let Some(oid) = &mapped
            && let Some(before) = self.own_earlier(oid)?
        {
            return self.restate(before, fresh);
        }
        self.refuse_unpermitted_or_unlabelled(create, None, &fresh)?;
        let landed = self.put_or_send_among(create, None, fresh, &|earlier| {
            Some(earlier.target()) == mapped.as_ref()
        })?;
        self.store.map_id(&key, landed.oid())?;
        Ok(landed)
    }

    pub fn caller_id(&self, oid: &Oid) -> Result<Option<CallerId>, UseCaseError> {
        Ok(self.store.id_for_oid(&self.id_holder(), oid)?)
    }

    fn id_holder(&self) -> IdHolder {
        match self.access.author() {
            Author::Owner => IdHolder::Owner,
            Author::Token(token) => IdHolder::Token(token.name),
        }
    }

    pub(crate) fn id_key(&self, id: &CallerId) -> IdKey {
        IdKey {
            holder: self.id_holder(),
            session: self
                .access
                .details()
                .and_then(|d| d.session())
                .map(str::to_string),
            id: id.clone(),
        }
    }

    fn own_earlier(&self, oid: &Oid) -> Result<Option<Object>, UseCaseError> {
        if let Some(object) = self.store.get(oid)?.filter(|o| self.access.sees(o)) {
            return Ok(Some(object));
        }
        Ok(self
            .store
            .item(&ItemHash::new(oid.clone()))?
            .filter(|item| {
                item.is_waiting() && self.access.token() == Some(item.author.name.as_str())
            })
            .and_then(|item| item.change.after().cloned()))
    }
}

mod restate;

#[cfg(test)]
mod tests;
