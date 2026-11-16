#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step<'a> {
    pub doing: &'a str,
    pub did: &'a str,
    pub unit: &'a str,
    pub total: Option<usize>,
}

pub trait Progress {
    fn start(&self, step: Step<'_>);
    fn advance(&self, done: usize);
    fn finish(&self);
    fn fail(&self);
}

pub struct NoProgress;

impl Progress for NoProgress {
    fn start(&self, _: Step<'_>) {}
    fn advance(&self, _: usize) {}
    fn finish(&self) {}
    fn fail(&self) {}
}
