//! Error types for the UniFi SDK.

use thiserror::Error;

/// Result type alias for UniFi SDK operations.
pub type Result<T> = std::result::Result<T, UnifiError>;

/// Errors that can occur when interacting with the UniFi controller.
#[derive(Debug, Error)]
pub enum UnifiError {
    /// Authentication failed (invalid credentials or session expired).
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Session has expired and needs re-authentication.
    #[error("session expired")]
    SessionExpired,

    /// CSRF token is missing or invalid.
    #[error("CSRF token error: {0}")]
    CsrfError(String),

    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Invalid URL provided.
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// API returned an error response.
    #[error("API error: {message} (code: {code})")]
    ApiError {
        /// Error code from the API.
        code: String,
        /// Error message from the API.
        message: String,
    },

    /// Resource not found.
    #[error("resource not found: {0}")]
    NotFound(String),

    /// Invalid response from the API.
    #[error("invalid response: {0}")]
    InvalidResponse(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Configuration error.
    #[error("configuration error: {0}")]
    ConfigError(String),

    /// Rate limited by the controller.
    #[error("rate limited, retry after {retry_after_secs:?} seconds")]
    RateLimited {
        /// Optional retry-after duration in seconds.
        retry_after_secs: Option<u64>,
    },

    /// Connection error (controller unreachable).
    #[error("connection error: {0}")]
    ConnectionError(String),

    /// TLS/certificate error.
    #[error("TLS error: {0}")]
    TlsError(String),

    /// Operation timed out.
    #[error("operation timed out")]
    Timeout,

    /// Invalid input provided.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Resource already exists.
    #[error("resource already exists: {0}")]
    AlreadyExists(String),

    /// Operation not supported on this controller version.
    #[error("operation not supported: {0}")]
    NotSupported(String),
}

impl UnifiError {
    /// Create an API error from code and message.
    #[must_use]
    pub fn api_error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ApiError { code: code.into(), message: message.into() }
    }

    /// Check if this error is retryable.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::SessionExpired
                | Self::RateLimited { .. }
                | Self::ConnectionError(_)
                | Self::Timeout
        )
    }

    /// Check if this error indicates authentication is needed.
    #[must_use]
    pub const fn needs_auth(&self) -> bool {
        matches!(self, Self::AuthenticationFailed(_) | Self::SessionExpired)
    }
}

/// API response wrapper for UniFi controller responses.
#[derive(Debug, serde::Deserialize)]
pub struct ApiResponse<T> {
    /// Response metadata.
    pub meta: ApiResponseMeta,
    /// Response data (if successful).
    pub data: Option<T>,
}

/// Metadata from API responses.
#[derive(Debug, serde::Deserialize)]
pub struct ApiResponseMeta {
    /// Response code (e.g., "ok", "error").
    pub rc: String,
    /// Optional error message.
    #[serde(default)]
    pub msg: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Check if the response indicates success.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.meta.rc == "ok"
    }

    /// Convert to a Result, extracting data on success or error on failure.
    pub fn into_result(self) -> Result<T> {
        if self.is_ok() {
            self.data.ok_or_else(|| {
                UnifiError::InvalidResponse("response marked ok but no data present".to_string())
            })
        } else {
            Err(UnifiError::api_error(
                &self.meta.rc,
                self.meta.msg.unwrap_or_else(|| "unknown error".to_string()),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_retryable() {
        assert!(UnifiError::SessionExpired.is_retryable());
        assert!(UnifiError::RateLimited { retry_after_secs: Some(60) }.is_retryable());
        assert!(UnifiError::ConnectionError("timeout".to_string()).is_retryable());
        assert!(UnifiError::Timeout.is_retryable());

        assert!(!UnifiError::AuthenticationFailed("bad password".to_string()).is_retryable());
        assert!(!UnifiError::NotFound("rule".to_string()).is_retryable());
    }

    #[test]
    fn test_error_needs_auth() {
        assert!(UnifiError::AuthenticationFailed("bad password".to_string()).needs_auth());
        assert!(UnifiError::SessionExpired.needs_auth());

        assert!(!UnifiError::NotFound("rule".to_string()).needs_auth());
        assert!(!UnifiError::Timeout.needs_auth());
    }

    #[test]
    fn test_api_response_ok() {
        let response: ApiResponse<Vec<String>> = ApiResponse {
            meta: ApiResponseMeta { rc: "ok".to_string(), msg: None },
            data: Some(vec!["test".to_string()]),
        };

        assert!(response.is_ok());
        let result = response.into_result().unwrap();
        assert_eq!(result, vec!["test".to_string()]);
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<Vec<String>> = ApiResponse {
            meta: ApiResponseMeta {
                rc: "error".to_string(),
                msg: Some("invalid request".to_string()),
            },
            data: None,
        };

        assert!(!response.is_ok());
        let err = response.into_result().unwrap_err();
        assert!(matches!(err, UnifiError::ApiError { .. }));
    }
}
