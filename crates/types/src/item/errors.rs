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
