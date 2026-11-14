use crate::access::Access;
use crate::config::Config;
use crate::ports::{Clock, CredentialSource, FilterFunctions, HelperLauncher, Randomness, Store};

pub struct Dam<'a> {
    pub store: &'a dyn Store,
    pub clock: &'a dyn Clock,
    pub random: &'a dyn Randomness,
    pub config: &'a Config,
    pub access: &'a dyn Access,
    pub filter_functions: &'a dyn FilterFunctions,
}

#[derive(Clone, Copy)]
pub struct Remotes<'a> {
    pub launcher: &'a dyn HelperLauncher,
    pub credentials: &'a dyn CredentialSource,
}
