use clap::Args;

const HEADING: &str = "Order and size";

#[derive(Args, Clone, Debug, Default)]
pub(crate) struct OrderArgs {
    #[arg(
        long,
        value_name = "FIELD[:desc]",
        help_heading = HEADING,
        help = "Sort by this field, smallest first, or largest with :desc; empty values last. Repeatable, or a,b"
    )]
    pub(crate) sort: Vec<String>,
    #[arg(long, value_name = "N", help_heading = HEADING, help = "List at most N rows")]
    pub(crate) limit: Option<usize>,
    #[arg(long, value_name = "N", help_heading = HEADING, help = "Skip the first N rows")]
    pub(crate) offset: Option<usize>,
}
