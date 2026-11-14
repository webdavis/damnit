use std::os::unix::fs::PermissionsExt;

use dam_adapters::{Interrupt, ProcessLauncher};

pub const FRESH_HELPER_PROCESSES_TRIED: usize = 8;

pub fn the_real_helper_against(base: &str, dir: &std::path::Path) -> ProcessLauncher {
    let wrapper = dir.join("dam-remote-todoist");
    let script = format!(
        "#!/bin/sh\nDAM_TODOIST_BASE_URL={base} exec {} \"$@\"\n",
        env!("CARGO_BIN_EXE_dam-remote-todoist")
    );
    std::fs::write(&wrapper, script).unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
    ProcessLauncher::with_search_path(dir.as_os_str(), Interrupt::default())
}
