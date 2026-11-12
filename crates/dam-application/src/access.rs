mod authenticate;

use dam_domain::{Author, Channel, Grants, Object, Permission, Subject, TokenAuthor};

pub use authenticate::{TokenRefusal, authenticate};

pub trait Access {
    fn token(&self) -> Option<&str>;
    fn author(&self) -> Author;
    fn holds(&self, permission: Permission) -> bool;
    fn permits(&self, permission: Permission, touched: &[Subject<'_>]) -> bool;

    fn sees(&self, object: &Object) -> bool {
        self.permits(Permission::read(object.kind()), &[Subject::Object(object)])
    }
}

pub struct Owner;

impl Access for Owner {
    fn token(&self) -> Option<&str> {
        None
    }

    fn author(&self) -> Author {
        Author::Owner
    }

    fn holds(&self, _: Permission) -> bool {
        true
    }

    fn permits(&self, _: Permission, _: &[Subject<'_>]) -> bool {
        true
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Holder {
    pub name: String,
    pub grants: Grants,
    pub channel: Channel,
}

impl Access for Holder {
    fn token(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn author(&self) -> Author {
        Author::Token(TokenAuthor {
            name: self.name.clone(),
            key_id: self.grants.holder.clone(),
            channel: self.channel,
        })
    }

    fn holds(&self, permission: Permission) -> bool {
        self.grants.holds(permission)
    }

    fn permits(&self, permission: Permission, touched: &[Subject<'_>]) -> bool {
        self.grants.permits(permission, touched)
    }
}
