use lambda_http::{Request, RequestPayloadExt, Response};
use serde::de::DeserializeOwned;

use crate::common::errors::{Error, Error::HttpError};

const EMPTY_PAYLOAD_ERROR: &str = "Request payload is empty";

pub fn extract_request<T: DeserializeOwned>(request: Request) -> Result<T, Error> {
    return match request.payload::<T>() {
        Ok(Some(val)) => Ok(val),
        Ok(None) => {
            let response = Response::builder()
                .status(400)
                .header("content-type", "text/html")
                .body(EMPTY_PAYLOAD_ERROR.into())
                .map_err(Box::new)?;

            return Err(HttpError(response));
        }
        Err(err) => {
            let response = Response::builder()
                .status(400)
                .header("content-type", "text/html")
                .body(err.to_string().into())
                .map_err(Box::new)?;

            Err(HttpError(response))
        }
    };
}
