//! The TOML template `dam edit -e` opens in the operator's editor, and the
//! field diff a saved one reads back as.

mod parse;
mod reader;
mod render;

pub use parse::parse_template;
pub use render::render_template;

#[cfg(test)]
mod tests;
