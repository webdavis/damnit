use dam_domain::Object;

use crate::config::RemoteConfig;
use crate::remote::RemoteCapabilities;
use crate::use_cases::push::owed::excepted::is_excepted;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skip {
    OutsidePath,
    Excepted,
    KindNotTaken,
    NoFieldItTakes,
    NeverSent,
}

impl Skip {
    pub fn as_str(self) -> &'static str {
        match self {
            Skip::OutsidePath => "outside_path",
            Skip::Excepted => "excepted",
            Skip::KindNotTaken => "kind_not_taken",
            Skip::NoFieldItTakes => "no_field_it_takes",
            Skip::NeverSent => "never_sent",
        }
    }
}

pub(super) fn not_going(
    remote: &RemoteConfig,
    caps: &RemoteCapabilities,
    object: &Object,
) -> Option<Skip> {
    if !caps.kinds.contains(&object.kind()) {
        return Some(Skip::KindNotTaken);
    }
    let within_its_path = remote
        .path
        .as_ref()
        .is_none_or(|scope| object.base().path.is_within(scope));
    if !within_its_path {
        return Some(Skip::OutsidePath);
    }
    is_excepted(remote, object).then_some(Skip::Excepted)
}
