use dam_domain::Path;

use crate::access::Access;
use crate::config::Config;
use crate::ports::{
    Clock, CredentialSource, EditRequests, FilterFunctions, HelperLauncher, HookEdits,
    HookWarnings, Hooks, Interruption, Progress, Randomness, Store,
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
    pub progress: &'a dyn Progress,
    pub edit_requests: &'a dyn EditRequests,
    pub hook_edits: &'a dyn HookEdits,
    pub interruption: &'a dyn Interruption,
    pub project_hooks: Option<ProjectHooks<'a>>,
}

#[derive(Clone, Copy)]
pub struct ProjectHooks<'a> {
    pub config: &'a Config,
    pub hooks: &'a dyn Hooks,
    pub within: &'a Path,
}

#[derive(Clone, Copy)]
pub struct Remotes<'a> {
    pub launcher: &'a dyn HelperLauncher,
    pub credentials: &'a dyn CredentialSource,
}
