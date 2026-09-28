mod parse;
mod reader;
mod render;

pub use parse::parse_template;
pub use render::render_template;

#[cfg(test)]
mod tests;
