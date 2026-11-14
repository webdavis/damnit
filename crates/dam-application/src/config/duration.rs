use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfiguredDuration {
    pub value: Duration,
    pub text: String,
}

impl ConfiguredDuration {
    pub fn parse(text: &str) -> Option<ConfiguredDuration> {
        let mut chars = text.chars();
        let unit = chars.next_back()?;
        let digits = Some(chars.as_str()).filter(|d| d.bytes().all(|b| b.is_ascii_digit()))?;
        let n: u64 = digits.parse().ok()?;
        let secs = match unit {
            's' => Some(n),
            'm' => n.checked_mul(60),
            'h' => n.checked_mul(3600),
            _ => None,
        }?;
        Some(ConfiguredDuration {
            value: Duration::from_secs(secs),
            text: text.to_string(),
        })
    }
}
