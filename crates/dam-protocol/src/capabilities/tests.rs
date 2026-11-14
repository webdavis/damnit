use super::*;

#[test]
fn the_current_version_and_every_older_one_is_supported() {
    for protocol in 1..=PROTOCOL_VERSION {
        assert!(caps(protocol).supported(), "{protocol}");
    }
}

#[test]
fn protocol_zero_never_existed_and_is_not_supported() {
    assert!(!caps(0).supported());
}

#[test]
fn one_version_past_the_current_one_is_not_supported() {
    assert!(!caps(PROTOCOL_VERSION + 1).supported());
}

fn caps(protocol: u32) -> Capabilities {
    Capabilities {
        protocol,
        kinds: vec![],
        fields: vec![],
        credentials: vec![],
        incremental: false,
    }
}
