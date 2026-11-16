use std::os::unix::fs::PermissionsExt;

use super::sandbox::Sandbox;

const HOLDS_NOTHING: [(&str, &str); 2] = [
    (
        "launchctl",
        "#!/bin/sh\n[ \"$1\" = print ] && exit 113\nexit 1\n",
    ),
    (
        "systemctl",
        "#!/bin/sh\n[ \"$2\" = is-active ] && exit 3\nexit 1\n",
    ),
];

impl Sandbox {
    pub(super) fn install_schedulers_that_hold_nothing(&self) {
        for (program, script) in HOLDS_NOTHING {
            let path = self.dir.path().join("bin").join(program);
            std::fs::write(&path, script).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
}
