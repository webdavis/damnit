use clap::Args;
use dam_domain::Path;

#[derive(Args, Debug, Default)]
pub(crate) struct SetArgs {
    #[arg(
        long,
        conflicts_with_all = ["oid", "filter"],
        help = "Act on every object this query lists; inside a project, only its tasks unless --global"
    )]
    pub(crate) query: Option<String>,
    #[arg(
        long,
        conflicts_with = "oid",
        help = "Act on every object this saved filter lists; inside a project, only its tasks unless --global"
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
    #[arg(
        long,
        help = "For a token: the changes wait as one set the owner takes or declines whole"
    )]
    pub(crate) as_one: bool,
    #[arg(skip)]
    pub(crate) place: Path,
}

impl SetArgs {
    pub(crate) fn is_plain(&self) -> bool {
        self.query.is_none()
            && self.filter.is_none()
            && !self.preview
            && self.expect.is_none()
            && !self.as_one
    }
}
