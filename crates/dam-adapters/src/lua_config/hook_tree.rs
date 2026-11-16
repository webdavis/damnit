use std::fmt::Write;
use std::path::{Path, PathBuf};

use dam_application::{ConfigError, HookEntry, HookFingerprint};

use super::hook_folder::unreadable;

const MOST_ENTRIES: usize = 1000;
const MOST_BYTES: u64 = 8 * 1024 * 1024;

pub(super) enum TreeEntry {
    File(Vec<u8>),
    Folder,
    Link(Vec<u8>),
}

#[derive(Default)]
pub(super) struct HookTree {
    pub(super) entries: Vec<(PathBuf, TreeEntry)>,
    pub(super) too_large: bool,
}

pub(super) fn read_tree(root: &Path) -> Result<HookTree, ConfigError> {
    let mut tree = HookTree::default();
    let mut bytes = 0;
    let mut pending = vec![PathBuf::new()];
    while let Some(folder) = pending.pop() {
        let entries = match std::fs::read_dir(root.join(&folder)) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && folder.as_os_str().is_empty() => {
                return Ok(tree);
            }
            other => other.map_err(|e| unreadable(&root.join(&folder), &e))?,
        };
        for entry in entries {
            let entry = entry.map_err(|e| unreadable(root, &e))?;
            let relative = folder.join(entry.file_name());
            let path = entry.path();
            let kind = entry.file_type().map_err(|e| unreadable(&path, &e))?;
            let kept = if kind.is_symlink() {
                let target = std::fs::read_link(&path).map_err(|e| unreadable(&path, &e))?;
                TreeEntry::Link(target.into_os_string().into_encoded_bytes())
            } else if kind.is_dir() {
                pending.push(relative.clone());
                TreeEntry::Folder
            } else {
                let size = entry.metadata().map_err(|e| unreadable(&path, &e))?.len();
                bytes += size;
                if bytes > MOST_BYTES {
                    return Ok(too_large());
                }
                TreeEntry::File(std::fs::read(&path).map_err(|e| unreadable(&path, &e))?)
            };
            tree.entries.push((relative, kept));
            if tree.entries.len() > MOST_ENTRIES {
                return Ok(too_large());
            }
        }
    }
    Ok(tree)
}

fn too_large() -> HookTree {
    HookTree {
        entries: Vec::new(),
        too_large: true,
    }
}

impl HookTree {
    pub(super) fn fingerprint(&self, folder_name: &str) -> Option<HookFingerprint> {
        if self.too_large {
            return None;
        }
        Some(HookFingerprint::of_tree(self.entries.iter().map(
            |(relative, entry)| {
                let name = format!("{folder_name}/{}", escaped(relative));
                match entry {
                    TreeEntry::File(bytes) => (name, HookEntry::File(bytes)),
                    TreeEntry::Folder => (name + "/", HookEntry::Folder),
                    TreeEntry::Link(target) => (name, HookEntry::Link(target)),
                }
            },
        )))
    }
}

fn escaped(relative: &Path) -> String {
    let mut name = String::new();
    for chunk in relative.as_os_str().as_encoded_bytes().utf8_chunks() {
        for c in chunk.valid().chars() {
            match c.is_control() || c == '\\' {
                true => name.extend(c.escape_unicode()),
                false => name.push(c),
            }
        }
        for byte in chunk.invalid() {
            let _ = write!(name, "\\x{byte:02x}");
        }
    }
    name
}
