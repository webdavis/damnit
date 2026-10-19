use crate::config::Config;
use crate::ports::{Clock, Randomness, Store};

pub struct Dam<'a> {
    pub store: &'a dyn Store,
    pub clock: &'a dyn Clock,
    pub random: &'a dyn Randomness,
    pub config: &'a Config,
}
