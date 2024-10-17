use lambda_http::http::StatusCode;
use lambda_http::{Request, RequestPayloadExt, Response};
use serde::Deserialize;
use serde_json::json;
use tracing::{error, info, instrument, trace};

use crate::common::errors::{Error, Error::HttpError};

const EMPTY_PAYLOAD_ERROR: &str = "Request payload is empty";

#[instrument(skip(request))]
pub fn extract_request<T>(request: &Request) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    trace!("extracting request");
    return match request.payload::<T>() {
        Ok(Some(val)) => Ok(val),
        Ok(None) => {
            info!(EMPTY_PAYLOAD_ERROR);
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(error_string_to_json(EMPTY_PAYLOAD_ERROR).to_string())
                .map_err(Box::new)?;

            return Err(HttpError(response));
        }
        Err(err) => {
            error!("Failed to deserialize payload: {}", err.to_string());
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(error_string_to_json(&err).to_string())
                .map_err(Box::new)?;

            Err(HttpError(response))
        }
    };
}

pub fn error_string_to_json<T: ToString + ?Sized>(error_str: &T) -> serde_json::Value {
    json!({
        "error": error_str.to_string()
    })
}
