pub type StdError = Box<dyn std::error::Error>;

#[derive(Debug)]
pub enum Error {
    IndustryDbError(Box<dyn std::error::Error>),
    Unimplemented,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
