#[derive(thiserror::Error, Debug)]
pub enum ItemError {
    #[error("Invalid Item format: {0}")]
    FormatError(String),
    #[error(transparent)]
    NumParseError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    DataParseError(#[from] chrono::format::ParseError),
}

impl ItemError {
    pub(crate) fn absent_attribute_error(attribute_name: &str) -> Self {
        let err_str = format!("No {} attribute in item", attribute_name);
        Self::FormatError(err_str)
    }

    pub(crate) fn invalid_attribute_type(attribute_name: &str, t: &str) -> Self {
        let err_str = format!("{} attribute value isn't a {}", attribute_name, t);
        Self::FormatError(err_str)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ServerError {
    UnsupportedCompilerVersion,
    CompilationError,
    InternalError,
}

impl Into<&'static str> for ServerError {
    fn into(self) -> &'static str {
        match self {
            ServerError::UnsupportedCompilerVersion => "UnsupportedCompilerVersion",
            ServerError::CompilationError => "CompilationError",
            ServerError::InternalError => "InternalError",
        }
    }
}

impl TryFrom<&str> for ServerError {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "UnsupportedCompilerVersion" => Ok(ServerError::UnsupportedCompilerVersion),
            "CompilationError" => Ok(ServerError::CompilationError),
            "InternalError" => Ok(ServerError::InternalError),
            _ => Err(value.into()),
        }
    }
}
