use dam_application::StoreError;
use dam_domain::{Author, Authored, Change, Channel, KeyId, TokenAuthor};
use rusqlite::Params;

use super::codec::{change_from_row, object_to_json};
use super::objects::sql;

pub(super) const AUTHORED_COLUMNS: &str =
    "op, before_json, after_json, author_token, author_key_id, author_channel";

pub(super) struct ChangeColumns {
    pub(super) op: &'static str,
    pub(super) before: Option<String>,
    pub(super) after: Option<String>,
}

pub(super) struct AuthorColumns<'a> {
    pub(super) token: Option<&'a str>,
    pub(super) key_id: Option<&'a str>,
    pub(super) channel: Option<&'static str>,
}

pub(super) fn change_columns(change: &Change) -> Result<ChangeColumns, StoreError> {
    Ok(ChangeColumns {
        op: change.op().as_str(),
        before: change.before().map(object_to_json).transpose()?,
        after: change.after().map(object_to_json).transpose()?,
    })
}

pub(super) fn author_columns(author: &Author) -> AuthorColumns<'_> {
    match author {
        Author::Owner => AuthorColumns {
            token: None,
            key_id: None,
            channel: None,
        },
        Author::Token(token) => token_columns(token),
    }
}

pub(super) fn token_columns(token: &TokenAuthor) -> AuthorColumns<'_> {
    AuthorColumns {
        token: Some(&token.name),
        key_id: Some(token.key_id.as_str()),
        channel: Some(token.channel.as_str()),
    }
}

pub(super) fn token_from_columns(
    name: String,
    key_id: &str,
    channel: &str,
) -> Result<TokenAuthor, StoreError> {
    Ok(TokenAuthor {
        name,
        key_id: KeyId::parse(key_id)
            .ok_or_else(|| StoreError::Failed(format!("stored key id {key_id:?}")))?,
        channel: Channel::parse(channel)
            .ok_or_else(|| StoreError::Failed(format!("stored channel {channel:?}")))?,
    })
}

fn author_from_columns(
    token: Option<String>,
    key_id: Option<String>,
    channel: Option<String>,
) -> Result<Author, StoreError> {
    match (token, key_id, channel) {
        (None, None, None) => Ok(Author::Owner),
        (Some(name), Some(key_id), Some(channel)) => {
            token_from_columns(name, &key_id, &channel).map(Author::Token)
        }
        _ => Err(StoreError::Failed("a stored author is half written".into())),
    }
}

type AuthoredRow = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

pub(super) fn read_authored(
    stmt: &mut rusqlite::Statement,
    params: impl Params,
) -> Result<Vec<Authored>, StoreError> {
    stmt.query_map(params, |r| {
        Ok((
            r.get(0)?,
            r.get(1)?,
            r.get(2)?,
            r.get(3)?,
            r.get(4)?,
            r.get(5)?,
        ))
    })
    .map_err(sql)?
    .map(|row| {
        let (op, before, after, token, key_id, channel): AuthoredRow = row.map_err(sql)?;
        Ok(Authored {
            change: change_from_row(&op, before, after)?,
            author: author_from_columns(token, key_id, channel)?,
        })
    })
    .collect()
}
