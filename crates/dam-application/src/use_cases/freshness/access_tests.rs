use std::time::Duration;

use dam_domain::Permission::{EventRead, RemoteRead};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::RemoteName;
use crate::service::Dam;
use crate::testing::{Scenario, allowed, config, holder};

#[test]
fn a_remote_the_token_may_not_read_is_answered_as_if_it_did_not_exist() {
    let mut config = config();
    config.remotes[0].name = RemoteName("gcal".into());
    let s = Scenario::default().configured(config);
    let bound = |name: &str| [(name.to_string(), Duration::from_secs(3600))];
    let token = holder(vec![allowed(&[EventRead], None)]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    for name in ["gcal", "nope"] {
        assert_eq!(
            dam.require_fresh(&bound(name)),
            Err(UseCaseError::Refused(Refusal::NoSuchRemote(name.into())))
        );
    }
    let reader = holder(vec![allowed(&[EventRead, RemoteRead], None)]);
    let dam = Dam {
        access: &reader,
        ..s.dam()
    };
    assert!(matches!(
        dam.require_fresh(&bound("gcal")),
        Err(UseCaseError::Refused(Refusal::StaleRemote { .. }))
    ));
}
