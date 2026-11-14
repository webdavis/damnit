use dam_adapters::{Interrupt, ProcessLauncher};

pub const FRESH_HELPER_PROCESSES_TRIED: usize = 8;

const WRAPPER_THAT_READS_ITS_FOLDER: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/helper_against_a_fake.sh"
);

pub fn the_real_helper_against(base: &str, dir: &std::path::Path) -> ProcessLauncher {
    std::fs::write(dir.join("base-url"), base).unwrap();
    std::fs::write(dir.join("helper"), env!("CARGO_BIN_EXE_dam-remote-todoist")).unwrap();
    std::os::unix::fs::symlink(
        WRAPPER_THAT_READS_ITS_FOLDER,
        dir.join("dam-remote-todoist"),
    )
    .unwrap();
    ProcessLauncher::with_search_path(dir.as_os_str(), Interrupt::default())
}
