use dam_domain::FilterTable;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FilterConfig {
    pub name: String,
    pub table: FilterTable,
    pub runs_a_function: bool,
}
