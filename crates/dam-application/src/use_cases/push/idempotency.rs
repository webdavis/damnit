use sha2::{Digest, Sha256};

use crate::ports::Declared;
use crate::remote::{MutationOp, RemoteMutation};

const UUID_VERSION_FOR_A_KEY_OF_OUR_OWN: char = '8';
const UUID_VARIANT: char = '8';
const ORDINAL_RESERVED_FOR_THE_HELPER: char = '0';

pub fn key(answers: u64, position: usize, write: &RemoteMutation, declared: &Declared) -> String {
    let mut hash = Sha256::new();
    hash.update(write.oid.as_str());
    hash.update(answers.to_be_bytes());
    match write.op {
        MutationOp::Create => hash.update("create"),
        MutationOp::Delete => {
            hash.update((position as u64).to_be_bytes());
            hash.update("delete");
        }
        MutationOp::Update => {
            hash.update((position as u64).to_be_bytes());
            hash.update("update");
            let into = match (&write.container_remote_id, &write.container_oid) {
                (Some(container), _) => format!("into {container}"),
                (None, Some(created)) => format!("into created {created}"),
                (None, None) => "in place".into(),
            };
            for part in std::iter::once(into.as_str()).chain(
                declared
                    .iter()
                    .flat_map(|(field, value)| [field.as_str(), value.as_str()]),
            ) {
                hash.update((part.len() as u64).to_be_bytes());
                hash.update(part);
            }
        }
    }
    let hex: String = hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{UUID_VERSION_FOR_A_KEY_OF_OUR_OWN}{}-{UUID_VARIANT}{}-{}{ORDINAL_RESERVED_FOR_THE_HELPER}",
        &hex[..8],
        &hex[8..12],
        &hex[12..15],
        &hex[15..18],
        &hex[18..29],
    )
}

#[cfg(test)]
mod tests;
