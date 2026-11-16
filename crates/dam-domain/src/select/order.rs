use super::Context;
use super::sort::Sort;
use crate::Object;

#[derive(Clone, Debug, Default)]
pub struct Order {
    pub sort: Sort,
    pub offset: usize,
    pub limit: Option<usize>,
}

impl Order {
    pub fn arrange(&self, objects: Vec<Object>, context: &dyn Context) -> Vec<Object> {
        let sorted = self.sort.arrange(objects, context).into_iter();
        let kept = sorted.skip(self.offset);
        match self.limit {
            Some(limit) => kept.take(limit).collect(),
            None => kept.collect(),
        }
    }
}
