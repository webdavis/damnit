mod base62;
mod checksum;

use std::fmt;

const PREFIX: &str = "damk_";
const KEY_ID_LEN: usize = 12;
const SECRET_LEN: usize = 32;
const RANDOM_CHARACTERS: usize = KEY_ID_LEN + SECRET_LEN;
const MAX_VERSION_DIGITS: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyId(String);

impl KeyId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone)]
pub struct Token {
    key_id: KeyId,
    text: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenError {
    NotAToken,
    UnknownVersion(String),
    Malformed,
    Checksum,
}

impl Token {
    pub fn parse(text: &str) -> Result<Token, TokenError> {
        let rest = text.strip_prefix(PREFIX).ok_or(TokenError::NotAToken)?;
        let (version, body) = rest.split_once('_').ok_or(TokenError::Malformed)?;
        match version {
            "v1" => parse_v1(text, body),
            other if is_a_version(other) => Err(TokenError::UnknownVersion(other.to_string())),
            _ => Err(TokenError::Malformed),
        }
    }

    pub fn generate(fill_with_entropy: &mut dyn FnMut(&mut [u8])) -> Token {
        let characters = base62::random_characters(RANDOM_CHARACTERS, fill_with_entropy);
        let (key_id, secret) = characters.split_at(KEY_ID_LEN);
        let body = format!("{PREFIX}v1_{key_id}_{secret}");
        let text = format!("{body}{}", checksum::of(&body));
        Token {
            key_id: KeyId(key_id.to_string()),
            text,
        }
    }

    pub fn key_id(&self) -> &KeyId {
        &self.key_id
    }

    pub fn expose(&self) -> String {
        self.text.clone()
    }

    pub fn matches(&self, other: &Token) -> bool {
        let (a, b) = (self.text.as_bytes(), other.text.as_bytes());
        let differing_bits = a
            .iter()
            .zip(b)
            .fold(0u8, |acc, (x, y)| std::hint::black_box(acc | (x ^ y)));
        a.len() == b.len() && differing_bits == 0
    }
}

fn is_a_version(word: &str) -> bool {
    word.strip_prefix('v').is_some_and(|digits| {
        (1..=MAX_VERSION_DIGITS).contains(&digits.len())
            && digits.chars().all(|c| c.is_ascii_digit())
    })
}

fn parse_v1(text: &str, body: &str) -> Result<Token, TokenError> {
    let (key_id, tail) = body.split_once('_').ok_or(TokenError::Malformed)?;
    let well_formed = key_id.len() == KEY_ID_LEN
        && tail.len() == SECRET_LEN + checksum::LEN
        && base62::all(key_id)
        && base62::all(tail);
    if !well_formed {
        return Err(TokenError::Malformed);
    }
    let (checked, sum) = text.split_at(text.len() - checksum::LEN);
    if checksum::of(checked) != sum {
        return Err(TokenError::Checksum);
    }
    Ok(Token {
        key_id: KeyId(key_id.to_string()),
        text: text.to_string(),
    })
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        self.matches(other)
    }
}

impl Eq for Token {}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token({}, <redacted>)", self.key_id)
    }
}

#[cfg(test)]
mod tests;
