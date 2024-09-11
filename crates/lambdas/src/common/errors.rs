use lambda_http::Response;

pub enum Error {
    HttpError(Response<String>),
    LambdaError(lambda_http::Error),
}

impl From<lambda_http::http::Error> for Error {
    fn from(value: lambda_http::http::Error) -> Self {
        Self::LambdaError(Box::new(value))
    }
}

impl<T> From<Box<T>> for Error
where
    T: std::error::Error + Send + Sync + 'static,
{
    fn from(value: Box<T>) -> Self {
        Self::LambdaError(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::LambdaError(Box::new(value))
    }
}
