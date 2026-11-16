#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectRuleFault {
    Shape,
    FilterIsAFunction,
    NotAQuery,
    LabelInTwoCategories,
    ClashesWithYourCategories,
}
