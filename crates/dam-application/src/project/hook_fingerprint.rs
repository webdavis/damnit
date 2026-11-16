use std::collections::BTreeMap;
use std::fmt::Write;

use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HookFingerprint(BTreeMap<String, String>);

pub enum HookEntry<'a> {
    File(&'a [u8]),
    Folder,
    Link(&'a [u8]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookFileChange {
    Changed,
    New,
    Removed,
}

impl HookFingerprint {
    pub fn of<'a>(files: impl IntoIterator<Item = (&'a str, &'a [u8])>) -> HookFingerprint {
        HookFingerprint::of_tree(
            files
                .into_iter()
                .map(|(name, bytes)| (name.to_string(), HookEntry::File(bytes))),
        )
    }

    pub fn of_tree<'a>(
        entries: impl IntoIterator<Item = (String, HookEntry<'a>)>,
    ) -> HookFingerprint {
        let kept = |entry| match entry {
            HookEntry::File(bytes) => sha256_hex(bytes),
            HookEntry::Folder => "folder".to_string(),
            HookEntry::Link(target) => format!("link {}", sha256_hex(target)),
        };
        HookFingerprint(
            entries
                .into_iter()
                .map(|(name, entry)| (name, kept(entry)))
                .collect(),
        )
    }

    pub fn from_files(files: BTreeMap<String, String>) -> HookFingerprint {
        HookFingerprint(files)
    }

    pub fn files(&self) -> &BTreeMap<String, String> {
        &self.0
    }

    pub fn changes(&self, now: &HookFingerprint) -> BTreeMap<String, HookFileChange> {
        let names = self.0.keys().chain(now.0.keys());
        names
            .filter_map(|name| {
                let change = match (self.0.get(name), now.0.get(name)) {
                    (Some(then), Some(now)) if then == now => return None,
                    (Some(_), Some(_)) => HookFileChange::Changed,
                    (None, _) => HookFileChange::New,
                    (_, None) => HookFileChange::Removed,
                };
                Some((name.clone(), change))
            })
            .collect()
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

#[cfg(test)]
mod tests;
