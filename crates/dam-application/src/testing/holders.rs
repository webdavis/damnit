use dam_domain::{Grant, Grants, Path, Permission, Token};

use crate::access::Holder;

pub fn holder(grants: Vec<Grant>) -> Holder {
    Holder {
        name: "vpt".into(),
        grants: Grants {
            holder: Token::generate(&mut |b: &mut [u8]| b.fill(1))
                .key_id()
                .clone(),
            grants,
        },
    }
}

pub fn allowed(permissions: &[Permission], paths: Option<&[&str]>) -> Grant {
    Grant {
        allow: permissions.iter().copied().collect(),
        paths: paths.map(|ps| ps.iter().map(|p| Path::parse(p).unwrap()).collect()),
        ..Grant::default()
    }
}
