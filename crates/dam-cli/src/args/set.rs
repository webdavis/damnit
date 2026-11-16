use clap::Args;

#[derive(Args, Debug, Default)]
pub(crate) struct SetArgs {
    #[arg(
        long,
        conflicts_with_all = ["oid", "filter"],
        help = "Act on every object this query lists, across all tasks even inside a project"
    )]
    pub(crate) query: Option<String>,
    #[arg(
        long,
        conflicts_with = "oid",
        help = "Act on every object this saved filter lists"
    )]
    pub(crate) filter: Option<String>,
    #[arg(long, help = "Show the changes and write nothing")]
    pub(crate) preview: bool,
    #[arg(
        long,
        value_name = "N",
        help = "Refuse, changing nothing, unless the set holds exactly N objects"
    )]
    pub(crate) expect: Option<usize>,
}

impl SetArgs {
    pub(crate) fn is_plain(&self) -> bool {
        self.query.is_none() && self.filter.is_none() && !self.preview && self.expect.is_none()
    }
}
