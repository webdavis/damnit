use dam_adapters::LuaConfigFile;
use dam_application::{Config, Hooks, ProjectHookSwitches, ProjectHooks};
use dam_domain::Path;

use crate::args::Command;
use crate::context::Context;
use crate::error::CliError;
use crate::project_scope::Standing;

pub(crate) struct ProjectHookSet {
    config: Config,
    hooks: Box<dyn Hooks>,
    within: Path,
}

struct ProjectHooksAsked<'a> {
    standing: &'a Standing,
    switches: &'a dyn ProjectHookSwitches,
    file: &'a LuaConfigFile,
    personal: &'a Config,
}

pub(crate) fn opened(
    ctx: Context,
    standing: &Standing,
    switches: &dyn ProjectHookSwitches,
    file: &LuaConfigFile,
    command: &Command,
) -> Result<Context, CliError> {
    let asked = ProjectHooksAsked {
        standing,
        switches,
        file,
        personal: &ctx.config,
    };
    let project_hooks = ProjectHookSet::turned_on(&asked, command)?;
    Ok(Context {
        project_hooks,
        ..ctx
    })
}

impl ProjectHookSet {
    fn turned_on(
        asked: &ProjectHooksAsked,
        command: &Command,
    ) -> Result<Option<ProjectHookSet>, CliError> {
        if reaches_beyond_the_project(command) {
            return Ok(None);
        }
        let Standing::Inside(project) = asked.standing else {
            return Ok(None);
        };
        let this_clone = asked.switches.switch(&project.folder)?;
        if !asked.personal.project_hooks_on(this_clone) {
            return Ok(None);
        }
        let Ok(within) = Path::parse(&project.settings.path) else {
            return Ok(None);
        };
        let owner = asked.personal.owner_name();
        let loaded = asked.file.project_hooks(&project.folder, owner)?;
        Ok(Some(ProjectHookSet {
            config: Config {
                hooks: loaded.hooks,
                task_pulled: loaded.task_pulled,
                ..asked.personal.clone()
            },
            hooks: loaded.run,
            within,
        }))
    }

    pub(crate) fn borrowed(&self) -> ProjectHooks<'_> {
        ProjectHooks {
            config: &self.config,
            hooks: self.hooks.as_ref(),
            within: &self.within,
        }
    }
}

fn reaches_beyond_the_project(command: &Command) -> bool {
    matches!(command, Command::New(a) if a.scope.global || a.scope.project.is_some())
}
