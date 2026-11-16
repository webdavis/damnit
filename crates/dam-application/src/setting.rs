mod key;
mod value;

pub use key::{KeyRefusal, SettingKey, SettingKind};
pub use value::{BadValue, SettingValue};

#[cfg(test)]
mod tests;
