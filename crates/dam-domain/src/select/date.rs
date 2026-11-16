use super::Context;
use super::moment::Moment;
use super::on_day::OnDay;
use crate::Timestamp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateTest(Shape);

#[derive(Clone, Debug, PartialEq, Eq)]
enum Shape {
    At(Moment),
    Between(Option<Moment>, Option<Moment>),
    On(OnDay),
}

impl DateTest {
    pub fn read(text: &str) -> Option<DateTest> {
        if let Some(day) = text.strip_prefix("on:") {
            return OnDay::read(day).map(|d| DateTest(Shape::On(d)));
        }
        let Some((from, to)) = text.split_once("..") else {
            return Moment::read(text).map(|m| DateTest(Shape::At(m)));
        };
        let end = |side: &str| match side {
            "" => Some(None),
            side => Moment::read(side).map(Some),
        };
        match (end(from)?, end(to)?) {
            (None, None) => None,
            (from, to) => Some(DateTest(Shape::Between(from, to))),
        }
    }

    pub fn holds(&self, point: Timestamp, context: &dyn Context) -> bool {
        let now = context.now();
        match &self.0 {
            Shape::At(moment) => moment.resolve(now).is_some_and(|s| s.holds(point)),
            Shape::Between(from, to) => {
                let after = from
                    .as_ref()
                    .is_none_or(|m| m.resolve(now).is_some_and(|s| s.from <= point));
                let before = to
                    .as_ref()
                    .is_none_or(|m| m.resolve(now).is_some_and(|s| point < s.end()));
                after && before
            }
            Shape::On(day) => day.holds(point.to_zoned(now.time_zone().clone()).date(), context),
        }
    }

    pub(crate) fn saved_filter(&self) -> Option<&str> {
        match &self.0 {
            Shape::On(OnDay::EventsOf(name)) => Some(name),
            _ => None,
        }
    }

    pub(crate) fn one_moment(&self) -> Option<&Moment> {
        match &self.0 {
            Shape::At(moment) if moment.is_one_day_or_instant() => Some(moment),
            _ => None,
        }
    }

    pub fn is_one_day_or_instant(&self) -> bool {
        matches!(&self.0, Shape::At(m) if m.is_one_day_or_instant())
    }
}
