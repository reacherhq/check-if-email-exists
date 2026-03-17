mod api;

mod headless;

use crate::util::ser_with_display::ser_with_display;
use reqwest::Error as ReqwestError;
use serde::Serialize;
use serde_json::error::Error as SerdeError;
use thiserror::Error;

pub use api::check_api;
pub use headless::check_headless;

/// Possible errors when checking Yahoo email addresses.
#[derive(Debug, Error, Serialize)]
pub enum YahooError {
	/// Cannot find "acrumb" field in cookie.
	#[error("Cannot find \"acrumb\" field in cookie")]
	NoAcrumb,
	/// Cannot find "sessionIndex" hidden input in body
	#[error("Cannot find \"sessionIndex\" hidden input in body")]
	NoSessionIndex,
	/// Cannot find cookie in Yahoo response.
	#[error("Cannot find cookie in Yahoo response")]
	NoCookie,
	/// Error when serializing or deserializing HTTP requests and responses.
	#[serde(serialize_with = "ser_with_display")]
	#[error("Error serializing or deserializing HTTP requests and responses: {0}")]
	ReqwestError(ReqwestError),
	/// Error when serializing or deserializing HTTP requests and responses.
	#[serde(serialize_with = "ser_with_display")]
	#[error("Error serializing or deserializing HTTP requests and responses: {0}")]
	SerdeError(SerdeError),
}

impl From<ReqwestError> for YahooError {
	fn from(error: ReqwestError) -> Self {
		YahooError::ReqwestError(error)
	}
}

impl From<SerdeError> for YahooError {
	fn from(error: SerdeError) -> Self {
		YahooError::SerdeError(error)
	}
}
