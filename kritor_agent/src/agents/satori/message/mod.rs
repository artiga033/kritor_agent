pub mod element;
mod parser;
pub use parser::Parser;

#[derive(Debug)]
pub struct Error(String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for Error {}
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
