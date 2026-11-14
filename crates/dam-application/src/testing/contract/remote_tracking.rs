use crate::ports::RemoteTrackingRepository;
use dam_domain::Timestamp;

use super::{oid, remote, task};

pub fn remote_tracking_repository_contract(tracking: &dyn RemoteTrackingRepository) {
    let here = remote("here");
    let there = remote("there");
    assert_eq!(tracking.remote_id(&here, &oid(1)).unwrap(), None);
    assert_eq!(tracking.oid_for_remote_id(&here, "r1").unwrap(), None);

    tracking.map_remote_id(&here, &oid(1), "r1").unwrap();
    assert_eq!(
        tracking.remote_id(&here, &oid(1)).unwrap(),
        Some("r1".to_string())
    );
    assert_eq!(
        tracking.oid_for_remote_id(&here, "r1").unwrap(),
        Some(oid(1))
    );
    assert_eq!(
        tracking.remote_id(&there, &oid(1)).unwrap(),
        None,
        "a mapping belongs to one remote"
    );

    tracking.map_remote_id(&here, &oid(1), "r2").unwrap();
    assert_eq!(
        tracking.remote_id(&here, &oid(1)).unwrap(),
        Some("r2".to_string()),
        "remapping an oid replaces its id"
    );

    let snapshot = task(1, "as the remote has it", "");
    tracking.set_remote_snapshot(&here, &snapshot).unwrap();
    assert_eq!(
        tracking.remote_snapshot(&here, &oid(1)).unwrap(),
        Some(snapshot)
    );
    assert_eq!(tracking.remote_snapshot(&there, &oid(1)).unwrap(), None);

    tracking.map_remote_id(&there, &oid(1), "t1").unwrap();
    tracking.clear_remote_mapping(&here, &oid(1)).unwrap();
    assert_eq!(tracking.remote_id(&here, &oid(1)).unwrap(), None);
    assert_eq!(tracking.remote_snapshot(&here, &oid(1)).unwrap(), None);
    assert_eq!(
        tracking.remote_id(&there, &oid(1)).unwrap(),
        Some("t1".to_string()),
        "clearing one remote's mapping leaves the other's standing"
    );

    assert_eq!(tracking.sync_token(&here).unwrap(), None);
    tracking.set_sync_token(&here, Some("cursor")).unwrap();
    assert_eq!(
        tracking.sync_token(&here).unwrap(),
        Some("cursor".to_string())
    );
    assert_eq!(tracking.sync_token(&there).unwrap(), None);
    tracking.set_sync_token(&here, None).unwrap();
    assert_eq!(
        tracking.sync_token(&here).unwrap(),
        None,
        "setting no token clears the one held"
    );

    assert_eq!(tracking.last_pull(&here).unwrap(), None);
    let at = Timestamp::UNIX_EPOCH;
    tracking.set_last_pull(&here, at).unwrap();
    assert_eq!(tracking.last_pull(&here).unwrap(), Some(at));
    assert_eq!(tracking.last_pull(&there).unwrap(), None);

    assert_eq!(tracking.last_push(&here).unwrap(), None);
    let pushed_at = at + std::time::Duration::from_secs(60);
    tracking.set_last_push(&here, pushed_at).unwrap();
    assert_eq!(tracking.last_push(&here).unwrap(), Some(pushed_at));
    assert_eq!(tracking.last_push(&there).unwrap(), None);
    assert_eq!(
        tracking.last_pull(&here).unwrap(),
        Some(at),
        "the two times are kept apart"
    );
}
