use std::path::Path;

use dam_adapters::LuaConfigFile;

use crate::output::Format;

pub(crate) fn create_the_owners_config_when_missing(
    file: &LuaConfigFile,
    path: &Path,
    format: Format,
) {
    let said = match file.create_if_missing() {
        Ok(false) => return,
        Ok(true) => format!(
            "dam: created {}, with every setting explained",
            path.display()
        ),
        Err(e) => format!("dam: could not create {}: {e}", path.display()),
    };
    if format == Format::Human {
        eprintln!("{said}");
    }
}
