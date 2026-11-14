use dam_domain::{CallerId, IdHolder, IdKey};

use super::oid;
use crate::ports::IdRepository;

fn key(holder: IdHolder, session: Option<&str>) -> IdKey {
    IdKey {
        holder,
        session: session.map(str::to_string),
        id: CallerId::parse("dentist").unwrap(),
    }
}

pub fn id_repository_contract(ids: &dyn IdRepository) {
    let tool = key(IdHolder::Token("tool".into()), None);
    assert_eq!(ids.oid_for_id(&tool).unwrap(), None);

    ids.map_id(&tool, &oid(1)).unwrap();
    assert_eq!(ids.oid_for_id(&tool).unwrap(), Some(oid(1)));
    assert_eq!(
        ids.id_for_oid(&tool.holder, &oid(1)).unwrap(),
        Some(tool.id.clone()),
        "the holder reads its id back from the object"
    );
    assert_eq!(ids.id_for_oid(&IdHolder::Owner, &oid(1)).unwrap(), None);
    for apart in [
        key(IdHolder::Owner, None),
        key(IdHolder::Token("claude".into()), None),
        key(IdHolder::Token("tool".into()), Some("s1")),
    ] {
        assert_eq!(ids.oid_for_id(&apart).unwrap(), None, "{apart:?}");
    }

    ids.map_id(&tool, &oid(2)).unwrap();
    assert_eq!(
        ids.oid_for_id(&tool).unwrap(),
        Some(oid(2)),
        "mapping an id again points it at the new object"
    );
}
