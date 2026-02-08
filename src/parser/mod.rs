mod reader;
mod writer;

pub use reader::parse_config;
pub use writer::write_config;

#[cfg(test)]
mod tests;
