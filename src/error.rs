use core::fmt;

#[derive(Debug)]
pub enum CliError {
    UnsupportedOS,
    Other { message: String },
}

impl std::error::Error for CliError {}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::UnsupportedOS => write!(f, "unsupported OS"),
            CliError::Other { message: _ } => write!(f, "unexpected error"),
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(value: std::io::Error) -> Self {
        CliError::Other {
            message: value.to_string(),
        }
    }
}
