mod parse;
mod reader;
mod render;

pub(crate) use parse::parse_template;
pub(crate) use render::render_template;

#[cfg(test)]
mod tests;
