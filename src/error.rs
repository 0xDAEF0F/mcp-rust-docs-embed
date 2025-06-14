use rmcp::Error as McpError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BackendError {
	#[error("could not resolve latest crate version for '{0}'")]
	VersionResolutionFailed(String),

	#[error(
		"no embedded documentation found for {crate_name} v{version}. Please run \
		 embed_docs first to generate and embed the documentation."
	)]
	NoEmbeddedDocs { crate_name: String, version: String },

	#[error("no results found for query: {0}")]
	NoQueryResults(String),

	#[error("no embedding operation found with ID: {0}")]
	OperationNotFound(String),

	#[error("failed to initialize service: {0}")]
	ServiceInitialization(String),

	#[error("internal error: {0}")]
	Internal(#[source] anyhow::Error),
}

impl From<BackendError> for McpError {
	fn from(err: BackendError) -> Self {
		use BackendError::*;

		match err {
			// user errors - invalid requests
			VersionResolutionFailed(_)
			| NoEmbeddedDocs { .. }
			| NoQueryResults(_)
			| OperationNotFound(_) => McpError::invalid_request(err.to_string(), None),
			// internal errors
			ServiceInitialization(_) | Internal(_) => {
				tracing::error!("Internal error: {:?}", err);
				McpError::internal_error("Internal server error", None)
			}
		}
	}
}
