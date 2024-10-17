use lambda_http::ext::PayloadError;
use lambda_http::{Request, RequestPayloadExt};
use serde::Deserialize;
use serde_json::json;
use tracing::{error,  instrument, trace};

#[derive(thiserror::Error, Debug)]
pub enum ExtractRequestError {
    #[error("Request payload is empty")]
    EmptyPayloadError,
    #[error("PayloadError: {0}")]
    PayloadError(#[from] PayloadError),
}

#[instrument(skip(request))]
pub fn extract_request<T>(request: &Request) -> Result<T, ExtractRequestError>
where
    T: for<'de> Deserialize<'de>,
{
    trace!("extracting request");
    let res = request
        .payload::<T>()?
        .ok_or(ExtractRequestError::EmptyPayloadError)?;
    Ok(res)
}

pub fn error_string_to_json<T: ToString + ?Sized>(error_str: &T) -> serde_json::Value {
    json!({
        "error": error_str.to_string()
    })
}
