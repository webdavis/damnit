use jiff::Zoned;

use crate::{Categories, Date, Oid};

pub trait Context {
    fn now(&self) -> &Zoned;
    fn categories(&self) -> &Categories;
    fn holds_up(&self, dependency: &Oid) -> bool;
    fn lists(&self, filter: &str, oid: &Oid) -> bool;
    fn has_event_on(&self, filter: &str, day: Date) -> bool;
    fn author(&self, oid: &Oid) -> &str;
}
