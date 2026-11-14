use sha2::{Digest, Sha256};

use crate::remote::{MutationOp, RemoteMutation};

const UUID_VERSION_FOR_A_KEY_OF_OUR_OWN: char = '8';
const UUID_VARIANT: char = '8';
const ORDINAL_RESERVED_FOR_THE_HELPER: char = '0';

pub fn key(answers: u64, write: &RemoteMutation, declared_on_the_wire: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(write.oid.as_str());
    hash.update(answers.to_be_bytes());
    hash.update(what_is_sent(write, declared_on_the_wire));
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

fn what_is_sent(write: &RemoteMutation, declared_on_the_wire: &str) -> String {
    match write.op {
        MutationOp::Create => "create".into(),
        MutationOp::Delete => "delete".into(),
        MutationOp::Update => {
            let into = match &write.container_remote_id {
                Some(container) => format!("into {container}"),
                None => "in place".into(),
            };
            format!("update\n{into}\n{declared_on_the_wire}")
        }
    }
}

#[cfg(test)]
mod tests;
