use clap::Args;

use crate::error::{CliError, quoted};

pub(crate) const FILTER_CREATE_HELP: &str = "The filter is saved as the table of the flags typed; dam list --filter NAME lists what it selects.

Examples:
  dam filter create today --due ..today
  dam filter create review --labels bug --labels not:waiting
  dam filter create hot --path work/ --any labels=urgent --any due=..today";

#[derive(Args, Debug, Default)]
pub(crate) struct OrderRefused {
    #[arg(long, hide = true)]
    sort: Vec<String>,
    #[arg(long, hide = true)]
    limit: Option<String>,
    #[arg(long, hide = true)]
    offset: Option<String>,
}

impl OrderRefused {
    pub(crate) fn refused(&self, name: &str) -> Option<CliError> {
        let mut words: Vec<String> = self
            .sort
            .iter()
            .map(|s| format!("--sort {}", quoted(s)))
            .collect();
        words.extend(self.limit.iter().map(|n| format!("--limit {}", quoted(n))));
        words.extend(
            self.offset
                .iter()
                .map(|n| format!("--offset {}", quoted(n))),
        );
        (!words.is_empty()).then(|| {
            CliError::Usage(format!(
                "a filter holds what to select, never an order or a size; save it without them, then run dam list --filter {} {}",
                quoted(name),
                words.join(" ")
            ))
        })
    }
}
