mod grammar;
mod writer;

pub use grammar::parse_config;
pub use writer::write_config;

#[cfg(test)]
mod tests;
