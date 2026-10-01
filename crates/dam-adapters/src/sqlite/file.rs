use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

use super::OpenError;

const OWNER_ONLY_FILE: u32 = 0o600;

pub(super) fn create_owner_only_in_one_call_so_no_symlink_can_be_planted_first(
    path: &Path,
) -> Result<(), OpenError> {
    match std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(OWNER_ONLY_FILE)
        .open(path)
    {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            refuse_anything_but_a_regular_file_then_make_it_owner_only(path)
        }
        Err(e) => Err(OpenError::Io(e.to_string())),
    }
}

fn refuse_anything_but_a_regular_file_then_make_it_owner_only(
    path: &Path,
) -> Result<(), OpenError> {
    let meta = std::fs::symlink_metadata(path).map_err(|e| OpenError::Io(e.to_string()))?;
    if !meta.file_type().is_file() {
        return Err(OpenError::Irregular(format!(
            "{} is not a regular file",
            path.display()
        )));
    }
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(OWNER_ONLY_FILE))
        .map_err(|e| OpenError::Io(e.to_string()))
}
