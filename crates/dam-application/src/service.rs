use crate::access::Access;
use crate::config::Config;
use crate::ports::{
    Clock, CredentialSource, EditRequests, FilterFunctions, HelperLauncher, HookWarnings, Hooks,
    Randomness, Store,
};

pub struct Dam<'a> {
    pub store: &'a dyn Store,
    pub clock: &'a dyn Clock,
    pub random: &'a dyn Randomness,
    pub config: &'a Config,
    pub access: &'a dyn Access,
    pub filter_functions: &'a dyn FilterFunctions,
    pub hooks: &'a dyn Hooks,
    pub hook_warnings: &'a dyn HookWarnings,
    pub edit_requests: &'a dyn EditRequests,
}

#[derive(Clone, Copy)]
pub struct Remotes<'a> {
    pub launcher: &'a dyn HelperLauncher,
    pub credentials: &'a dyn CredentialSource,
}
