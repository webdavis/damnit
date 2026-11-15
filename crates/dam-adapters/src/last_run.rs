use std::fs::{DirBuilder, Permissions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::Path;

pub fn replace_last_run(path: &Path, text: &str) -> std::io::Result<()> {
    let folder = path.parent().unwrap_or(Path::new("."));
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(folder)?;
    let mut file = tempfile::Builder::new()
        .permissions(Permissions::from_mode(0o600))
        .tempfile_in(folder)?;
    file.write_all(text.as_bytes())?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|e| e.error)?;
    Ok(())
}
