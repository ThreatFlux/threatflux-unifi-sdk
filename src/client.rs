//! UniFi controller client implementation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use reqwest::{
    Client, Response, StatusCode,
    header::{CONTENT_TYPE, COOKIE, HeaderMap, HeaderValue, SET_COOKIE},
};
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::RwLock;
use tracing::{debug, instrument, warn};
use url::Url;

use crate::error::{ApiResponse, Result, UnifiError};

/// Default timeout for HTTP requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default site name.
const DEFAULT_SITE: &str = "default";

/// Configuration for the UniFi client.
#[derive(Debug, Clone)]
pub struct UnifiConfig {
    /// Base URL of the UniFi controller (e.g., `https://192.168.1.1`).
    pub host: String,
    /// Username for authentication.
    pub username: String,
    /// Password for authentication.
    pub password: String,
    /// Site name (defaults to "default").
    pub site: String,
    /// Whether to verify TLS certificates (default: false for self-signed).
    pub verify_ssl: bool,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
}

impl UnifiConfig {
    /// Create a new configuration with required fields.
    #[must_use]
    pub fn new(
        host: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            host: host.into(),
            username: username.into(),
            password: password.into(),
            site: DEFAULT_SITE.to_string(),
            verify_ssl: false,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }

    /// Set the site name.
    #[must_use]
    pub fn with_site(mut self, site: impl Into<String>) -> Self {
        self.site = site.into();
        self
    }

    /// Enable or disable SSL verification.
    #[must_use]
    pub const fn with_verify_ssl(mut self, verify: bool) -> Self {
        self.verify_ssl = verify;
        self
    }

    /// Set the request timeout.
    #[must_use]
    pub const fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

/// Session state for the UniFi client.
#[derive(Debug, Default)]
struct SessionState {
    /// CSRF token for mutations.
    csrf_token: Option<String>,
    /// Whether we're currently authenticated.
    authenticated: bool,
    /// Session cookies (name -> value).
    cookies: HashMap<String, String>,
}

/// Controller type detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerType {
    /// UDM Pro / UniFi OS based controller.
    UnifiOs,
    /// Cloud Key or self-hosted controller.
    Classic,
}

/// Client for interacting with UniFi controllers.
#[derive(Debug)]
pub struct UnifiClient {
    /// HTTP client with cookie store.
    client: Client,
    /// Configuration.
    config: UnifiConfig,
    /// Base URL parsed.
    base_url: Url,
    /// Session state.
    session: Arc<RwLock<SessionState>>,
    /// Detected controller type.
    controller_type: RwLock<Option<ControllerType>>,
}

impl UnifiClient {
    /// Create a new UniFi client and authenticate.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection or authentication fails.
    #[instrument(skip(config), fields(host = %config.host, site = %config.site))]
    pub async fn connect(config: UnifiConfig) -> Result<Self> {
        let base_url = Self::parse_base_url(&config.host)?;

        let client = Client::builder()
            .danger_accept_invalid_certs(!config.verify_ssl)
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(UnifiError::HttpError)?;

        let unifi = Self {
            client,
            config,
            base_url,
            session: Arc::new(RwLock::new(SessionState::default())),
            controller_type: RwLock::new(None),
        };

        unifi.login().await?;
        Ok(unifi)
    }

    /// Create a client without connecting (for testing).
    pub fn new_disconnected(config: UnifiConfig) -> Result<Self> {
        let base_url = Self::parse_base_url(&config.host)?;

        let client = Client::builder()
            .danger_accept_invalid_certs(!config.verify_ssl)
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(UnifiError::HttpError)?;

        Ok(Self {
            client,
            config,
            base_url,
            session: Arc::new(RwLock::new(SessionState::default())),
            controller_type: RwLock::new(None),
        })
    }

    /// Parse and validate the base URL.
    fn parse_base_url(host: &str) -> Result<Url> {
        let url_str = if host.starts_with("http://") || host.starts_with("https://") {
            host.to_string()
        } else {
            format!("https://{host}")
        };

        Url::parse(&url_str).map_err(UnifiError::InvalidUrl)
    }

    /// Get the current site name.
    #[must_use]
    pub fn site(&self) -> &str {
        &self.config.site
    }

    /// Get the current configuration.
    #[must_use]
    pub fn config(&self) -> &UnifiConfig {
        &self.config
    }

    /// Change the active site.
    pub fn set_site(&mut self, site: impl Into<String>) {
        self.config.site = site.into();
    }

    /// Get the detected controller type.
    pub async fn controller_type(&self) -> Option<ControllerType> {
        *self.controller_type.read().await
    }

    /// Check if the client is authenticated.
    pub async fn is_authenticated(&self) -> bool {
        self.session.read().await.authenticated
    }

    /// Authenticate with the controller.
    #[instrument(skip(self))]
    pub async fn login(&self) -> Result<()> {
        debug!("Attempting login to UniFi controller");

        // Detect controller type and get login URL
        let controller_type = self.detect_controller_type().await?;

        let login_url = match controller_type {
            ControllerType::UnifiOs => self.base_url.join("/api/auth/login")?,
            ControllerType::Classic => self.base_url.join("/api/login")?,
        };

        let login_payload = serde_json::json!({
            "username": self.config.username,
            "password": self.config.password,
        });

        let response = self
            .client
            .post(login_url)
            .header(CONTENT_TYPE, "application/json")
            .json(&login_payload)
            .send()
            .await
            .map_err(|e| UnifiError::ConnectionError(e.to_string()))?;

        let status = response.status();

        // Extract CSRF token from response headers or cookies
        self.extract_csrf_token(&response).await;

        if status == StatusCode::OK || status == StatusCode::NO_CONTENT {
            let mut session = self.session.write().await;
            session.authenticated = true;
            *self.controller_type.write().await = Some(controller_type);
            debug!("Login successful, controller type: {:?}", controller_type);
            Ok(())
        } else if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            Err(UnifiError::AuthenticationFailed("Invalid username or password".to_string()))
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(UnifiError::AuthenticationFailed(format!(
                "Login failed with status {status}: {body}"
            )))
        }
    }

    /// Logout from the controller.
    #[instrument(skip(self))]
    pub async fn logout(&self) -> Result<()> {
        let controller_type = self.controller_type().await;
        let logout_url = match controller_type {
            Some(ControllerType::UnifiOs) => self.base_url.join("/api/auth/logout")?,
            _ => self.base_url.join("/api/logout")?,
        };

        let _ = self.client.post(logout_url).send().await;

        let mut session = self.session.write().await;
        session.authenticated = false;
        session.csrf_token = None;
        session.cookies.clear();

        debug!("Logged out from UniFi controller");
        Ok(())
    }

    /// Detect the controller type (UniFi OS vs Classic).
    async fn detect_controller_type(&self) -> Result<ControllerType> {
        // Try UniFi OS endpoint first (UDM Pro, etc.)
        let unifi_os_url = self.base_url.join("/api/auth/login")?;
        let response = self
            .client
            .get(unifi_os_url)
            .send()
            .await
            .map_err(|e| UnifiError::ConnectionError(e.to_string()))?;

        // UniFi OS returns 405 Method Not Allowed for GET on login
        // Classic controller returns 404
        if response.status() == StatusCode::METHOD_NOT_ALLOWED {
            Ok(ControllerType::UnifiOs)
        } else {
            Ok(ControllerType::Classic)
        }
    }

    /// Extract CSRF token and cookies from response headers.
    async fn extract_session_data(&self, response: &Response) {
        let mut session = self.session.write().await;

        // Try X-CSRF-Token header first
        if let Some(token) = response.headers().get("x-csrf-token")
            && let Ok(token_str) = token.to_str()
        {
            session.csrf_token = Some(token_str.to_string());
            debug!("Extracted CSRF token from header");
        }

        // Extract cookies from Set-Cookie headers
        for cookie in response.headers().get_all(SET_COOKIE) {
            if let Ok(cookie_str) = cookie.to_str() {
                // Parse cookie name=value from "name=value; path=/; ..."
                if let Some(name_value) = cookie_str.split(';').next()
                    && let Some((name, value)) = name_value.split_once('=')
                {
                    let name = name.trim().to_string();
                    let value = value.trim().to_string();

                    // Also extract CSRF token if present in cookie
                    if name == "csrf_token" || name == "TOKEN" {
                        session.csrf_token = Some(value.clone());
                        debug!("Extracted CSRF token from cookie: {}", name);
                    }

                    session.cookies.insert(name, value);
                }
            }
        }
    }

    /// Legacy method name for compatibility.
    async fn extract_csrf_token(&self, response: &Response) {
        self.extract_session_data(response).await;
    }

    /// Build the API URL for a given path.
    pub fn api_url(&self, path: &str) -> Result<Url> {
        let controller_type = futures::executor::block_on(async {
            self.controller_type().await.unwrap_or(ControllerType::UnifiOs)
        });

        let full_path = match controller_type {
            ControllerType::UnifiOs => {
                format!(
                    "/proxy/network/api/s/{}/{}",
                    self.config.site,
                    path.trim_start_matches('/')
                )
            }
            ControllerType::Classic => {
                format!("/api/s/{}/{}", self.config.site, path.trim_start_matches('/'))
            }
        };

        self.base_url.join(&full_path).map_err(UnifiError::InvalidUrl)
    }

    /// Build the API URL for a given path (async version).
    pub async fn api_url_async(&self, path: &str) -> Result<Url> {
        let controller_type = self.controller_type().await.unwrap_or(ControllerType::UnifiOs);

        let full_path = match controller_type {
            ControllerType::UnifiOs => {
                format!(
                    "/proxy/network/api/s/{}/{}",
                    self.config.site,
                    path.trim_start_matches('/')
                )
            }
            ControllerType::Classic => {
                format!("/api/s/{}/{}", self.config.site, path.trim_start_matches('/'))
            }
        };

        self.base_url.join(&full_path).map_err(UnifiError::InvalidUrl)
    }

    /// Build request headers including CSRF token and cookies.
    async fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let session = self.session.read().await;

        // Add CSRF token header
        if let Some(ref token) = session.csrf_token
            && let Ok(value) = HeaderValue::from_str(token)
        {
            headers.insert("x-csrf-token", value);
        }

        // Build Cookie header from stored cookies
        if !session.cookies.is_empty() {
            let cookie_str: String = session
                .cookies
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("; ");

            if let Ok(value) = HeaderValue::from_str(&cookie_str) {
                headers.insert(COOKIE, value);
            }
        }

        headers
    }

    /// Perform a GET request to the API.
    #[instrument(skip(self), fields(path = %path))]
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.ensure_authenticated().await?;

        let url = self.api_url_async(path).await?;
        let headers = self.build_headers().await;

        debug!("GET {}", url);

        let response = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| UnifiError::ConnectionError(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Perform a GET request to the API with query parameters.
    #[instrument(skip(self, query), fields(path = %path))]
    pub async fn get_with_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T> {
        self.ensure_authenticated().await?;

        let mut url = self.api_url_async(path).await?;
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query {
                pairs.append_pair(key, value);
            }
        }
        let headers = self.build_headers().await;

        debug!("GET {}?{}", url.path(), url.query().unwrap_or_default());

        let response = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| UnifiError::ConnectionError(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Perform a POST request to the API.
    #[instrument(skip(self, body), fields(path = %path))]
    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        self.ensure_authenticated().await?;

        let url = self.api_url_async(path).await?;
        let headers = self.build_headers().await;

        debug!("POST {}", url);

        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .map_err(|e| UnifiError::ConnectionError(e.to_string()))?;

        // Update CSRF token from response
        self.extract_csrf_token(&response).await;

        self.handle_response(response).await
    }

    /// Perform a PUT request to the API.
    #[instrument(skip(self, body), fields(path = %path))]
    pub async fn put<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        self.ensure_authenticated().await?;

        let url = self.api_url_async(path).await?;
        let headers = self.build_headers().await;

        debug!("PUT {}", url);

        let response = self
            .client
            .put(url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .map_err(|e| UnifiError::ConnectionError(e.to_string()))?;

        self.extract_csrf_token(&response).await;

        self.handle_response(response).await
    }

    /// Perform a DELETE request to the API.
    #[instrument(skip(self), fields(path = %path))]
    pub async fn delete(&self, path: &str) -> Result<()> {
        self.ensure_authenticated().await?;

        let url = self.api_url_async(path).await?;
        let headers = self.build_headers().await;

        debug!("DELETE {}", url);

        let response = self
            .client
            .delete(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| UnifiError::ConnectionError(e.to_string()))?;

        self.extract_csrf_token(&response).await;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else if status == StatusCode::NOT_FOUND {
            Err(UnifiError::NotFound("Resource not found".to_string()))
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(UnifiError::api_error(status.as_str(), body))
        }
    }

    /// Perform a command request (POST to /cmd/{manager}).
    #[instrument(skip(self, body), fields(manager = %manager))]
    pub async fn command<T: DeserializeOwned, B: Serialize>(
        &self,
        manager: &str,
        body: &B,
    ) -> Result<T> {
        let path = format!("cmd/{manager}");
        self.post(&path, body).await
    }

    /// Ensure we're authenticated, re-authenticate if needed.
    async fn ensure_authenticated(&self) -> Result<()> {
        if !self.is_authenticated().await {
            warn!("Session not authenticated, attempting re-login");
            self.login().await?;
        }
        Ok(())
    }

    /// Handle API response and extract data.
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();

        // Check for auth errors
        if status == StatusCode::UNAUTHORIZED {
            let mut session = self.session.write().await;
            session.authenticated = false;
            return Err(UnifiError::SessionExpired);
        }

        if status == StatusCode::FORBIDDEN {
            return Err(UnifiError::AuthenticationFailed("Access forbidden".to_string()));
        }

        if status == StatusCode::NOT_FOUND {
            return Err(UnifiError::NotFound("Resource not found".to_string()));
        }

        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            return Err(UnifiError::RateLimited { retry_after_secs: retry_after });
        }

        // Parse response body
        let body = response.text().await.map_err(|e| UnifiError::InvalidResponse(e.to_string()))?;

        if !status.is_success() {
            return Err(UnifiError::api_error(status.as_str(), body));
        }

        // Try to parse as API response wrapper first
        if let Ok(api_response) = serde_json::from_str::<ApiResponse<T>>(&body) {
            return api_response.into_result();
        }

        // Fall back to direct deserialization
        serde_json::from_str(&body).map_err(|e| {
            UnifiError::InvalidResponse(format!("Failed to parse response: {e}, body: {body}"))
        })
    }

    /// Get raw response for custom handling.
    pub async fn get_raw(&self, path: &str) -> Result<String> {
        self.ensure_authenticated().await?;

        let url = self.api_url_async(path).await?;
        let headers = self.build_headers().await;

        let response = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| UnifiError::ConnectionError(e.to_string()))?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            let mut session = self.session.write().await;
            session.authenticated = false;
            return Err(UnifiError::SessionExpired);
        }

        response.text().await.map_err(|e| UnifiError::InvalidResponse(e.to_string()))
    }

    /// Get raw response for custom handling with query parameters.
    pub async fn get_raw_with_query(&self, path: &str, query: &[(&str, String)]) -> Result<String> {
        self.ensure_authenticated().await?;

        let mut url = self.api_url_async(path).await?;
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query {
                pairs.append_pair(key, value);
            }
        }
        let headers = self.build_headers().await;

        let response = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| UnifiError::ConnectionError(e.to_string()))?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            let mut session = self.session.write().await;
            session.authenticated = false;
            return Err(UnifiError::SessionExpired);
        }

        response.text().await.map_err(|e| UnifiError::InvalidResponse(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = UnifiConfig::new("192.168.1.1", "admin", "password")
            .with_site("mysite")
            .with_verify_ssl(true)
            .with_timeout(60);

        assert_eq!(config.host, "192.168.1.1");
        assert_eq!(config.username, "admin");
        assert_eq!(config.password, "password");
        assert_eq!(config.site, "mysite");
        assert!(config.verify_ssl);
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn test_parse_base_url() {
        // With scheme
        let url = UnifiClient::parse_base_url("https://192.168.1.1").unwrap();
        assert_eq!(url.as_str(), "https://192.168.1.1/");

        // Without scheme (should default to https)
        let url = UnifiClient::parse_base_url("192.168.1.1").unwrap();
        assert_eq!(url.as_str(), "https://192.168.1.1/");

        // With port
        let url = UnifiClient::parse_base_url("192.168.1.1:8443").unwrap();
        assert_eq!(url.as_str(), "https://192.168.1.1:8443/");
    }

    #[test]
    fn test_config_defaults() {
        let config = UnifiConfig::new("host", "user", "pass");
        assert_eq!(config.site, "default");
        assert!(!config.verify_ssl);
        assert_eq!(config.timeout_secs, 30);
    }
}
