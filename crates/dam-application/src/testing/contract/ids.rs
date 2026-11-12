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
    let vpt = key(IdHolder::Token("vpt".into()), None);
    assert_eq!(ids.oid_for_id(&vpt).unwrap(), None);

    ids.map_id(&vpt, &oid(1)).unwrap();
    assert_eq!(ids.oid_for_id(&vpt).unwrap(), Some(oid(1)));
    assert_eq!(
        ids.id_for_oid(&vpt.holder, &oid(1)).unwrap(),
        Some(vpt.id.clone()),
        "the holder reads its id back from the object"
    );
    assert_eq!(ids.id_for_oid(&IdHolder::Owner, &oid(1)).unwrap(), None);
    for apart in [
        key(IdHolder::Owner, None),
        key(IdHolder::Token("claude".into()), None),
        key(IdHolder::Token("vpt".into()), Some("s1")),
    ] {
        assert_eq!(ids.oid_for_id(&apart).unwrap(), None, "{apart:?}");
    }

    ids.map_id(&vpt, &oid(2)).unwrap();
    assert_eq!(
        ids.oid_for_id(&vpt).unwrap(),
        Some(oid(2)),
        "mapping an id again points it at the new object"
    );
}
