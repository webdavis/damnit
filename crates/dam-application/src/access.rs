mod authenticate;

use dam_domain::{Grants, Object, Permission, Subject};

pub use authenticate::{TokenRefusal, authenticate};

pub trait Access {
    fn token(&self) -> Option<&str>;
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
}

impl Access for Holder {
    fn token(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn holds(&self, permission: Permission) -> bool {
        self.grants.holds(permission)
    }

    fn permits(&self, permission: Permission, touched: &[Subject<'_>]) -> bool {
        self.grants.permits(permission, touched)
    }
}
