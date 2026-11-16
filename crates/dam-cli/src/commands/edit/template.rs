mod parse;
mod reader;
mod render;

pub(crate) use parse::parse_template;
pub(crate) use render::render_template;

#[cfg(test)]
mod date_tests;
#[cfg(test)]
mod tests;
