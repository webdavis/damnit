use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use dam_domain::{Channel, Details, Timestamp};

pub fn process_details(meta: BTreeMap<String, String>, channel: Channel, at: Timestamp) -> Details {
    let dir = std::env::current_dir().ok();
    let checkout = dir.as_deref().and_then(enclosing_checkout);
    Details {
        meta,
        host: host_name(),
        dir: dir.as_deref().map(shown),
        repo: checkout.as_ref().map(|(root, _)| shown(root)),
        branch: checkout.and_then(|(_, branch)| branch),
        at,
        channel,
    }
}

fn shown(path: &Path) -> String {
    path.display().to_string()
}

fn host_name() -> Option<String> {
    let mut buffer = [0u8; 256];
    let status = unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) };
    if status != 0 {
        return None;
    }
    let end = buffer.iter().position(|b| *b == 0).unwrap_or(buffer.len());
    String::from_utf8(buffer[..end].to_vec())
        .ok()
        .filter(|name| !name.is_empty())
}

fn enclosing_checkout(dir: &Path) -> Option<(PathBuf, Option<String>)> {
    dir.ancestors().find_map(|root| {
        let marker = root.join(".git");
        let git_dir = if marker.is_dir() {
            marker
        } else {
            let text = std::fs::read_to_string(&marker).ok()?;
            root.join(text.strip_prefix("gitdir:")?.trim())
        };
        Some((root.to_path_buf(), branch_in(&git_dir)))
    })
}

fn branch_in(git_dir: &Path) -> Option<String> {
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    head.trim()
        .strip_prefix("ref: refs/heads/")
        .map(str::to_string)
}
