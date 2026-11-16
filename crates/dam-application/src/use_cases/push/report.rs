use dam_domain::Oid;

use crate::ports::RemoteName;

#[derive(Debug)]
pub struct PushReport {
    pub remote: RemoteName,
    pub sent: usize,
    pub succeeded: usize,
    pub failed: Vec<(Oid, String)>,
    pub skipped: usize,
    pub held: Option<usize>,
    pub unseen_failed: bool,
}
