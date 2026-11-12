use dam_domain::{Author, Details, Permission, Subject};

use super::Access;

pub struct Detailed {
    pub caller: Box<dyn Access>,
    pub details: Details,
}

impl Access for Detailed {
    fn token(&self) -> Option<&str> {
        self.caller.token()
    }

    fn author(&self) -> Author {
        self.caller.author()
    }

    fn holds(&self, permission: Permission) -> bool {
        self.caller.holds(permission)
    }

    fn permits(&self, permission: Permission, touched: &[Subject<'_>]) -> bool {
        self.caller.permits(permission, touched)
    }

    fn details(&self) -> Option<&Details> {
        Some(&self.details)
    }
}
