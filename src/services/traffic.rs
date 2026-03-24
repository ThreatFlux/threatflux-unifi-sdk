//! Traffic rule management service.

use tracing::instrument;

use crate::client::UnifiClient;
use crate::error::{Result, UnifiError};
use crate::models::traffic::TrafficRule;
use crate::types::Timeframe;

/// Service for managing traffic rules.
pub struct TrafficService<'a> {
    client: &'a UnifiClient,
}

impl<'a> TrafficService<'a> {
    /// Create a new traffic service.
    #[must_use]
    pub const fn new(client: &'a UnifiClient) -> Self {
        Self { client }
    }

    /// List all traffic rules.
    #[instrument(skip(self))]
    pub async fn list(&self) -> Result<Vec<TrafficRule>> {
        self.client.get("rest/trafficrule").await
    }

    /// Get a traffic rule by ID.
    #[instrument(skip(self))]
    pub async fn get(&self, id: &str) -> Result<TrafficRule> {
        let rules: Vec<TrafficRule> = self.client.get(&format!("rest/trafficrule/{id}")).await?;
        rules
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::NotFound(format!("Traffic rule {id} not found")))
    }

    /// Create a traffic rule.
    #[instrument(skip(self, rule))]
    pub async fn create(&self, rule: &TrafficRule) -> Result<TrafficRule> {
        let rules: Vec<TrafficRule> = self.client.post("rest/trafficrule", rule).await?;
        rules
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No rule returned".to_string()))
    }

    /// Update a traffic rule.
    #[instrument(skip(self, rule))]
    pub async fn update(&self, id: &str, rule: &TrafficRule) -> Result<TrafficRule> {
        let rules: Vec<TrafficRule> =
            self.client.put(&format!("rest/trafficrule/{id}"), rule).await?;
        rules
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No rule returned".to_string()))
    }

    /// Delete a traffic rule.
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("rest/trafficrule/{id}")).await
    }

    /// Enable or disable a traffic rule.
    #[instrument(skip(self))]
    pub async fn set_enabled(&self, id: &str, enabled: bool) -> Result<TrafficRule> {
        let mut rule = self.get(id).await?;
        rule.enabled = enabled;
        self.update(id, &rule).await
    }

    /// Create a domain blocking rule.
    #[instrument(skip(self))]
    pub async fn block_domains(
        &self,
        description: &str,
        domains: Vec<String>,
    ) -> Result<TrafficRule> {
        let rule = TrafficRule::block(description).for_all_clients().blocking_domains(domains);
        self.create(&rule).await
    }

    /// Create an IP blocking rule.
    #[instrument(skip(self))]
    pub async fn block_ips(&self, description: &str, ips: Vec<String>) -> Result<TrafficRule> {
        let rule = TrafficRule::block(description).for_all_clients().blocking_ips(ips);
        self.create(&rule).await
    }

    /// Create a rate limit rule for a client.
    #[instrument(skip(self))]
    pub async fn rate_limit_client(
        &self,
        description: &str,
        mac: &str,
        download_kbps: u32,
        upload_kbps: u32,
    ) -> Result<TrafficRule> {
        let rule = TrafficRule::rate_limit(description, download_kbps, upload_kbps).for_client(mac);
        self.create(&rule).await
    }

    /// Get traffic statistics for a timeframe.
    #[instrument(skip(self))]
    pub async fn get_stats(&self, timeframe: Timeframe) -> Result<Vec<serde_json::Value>> {
        let params = [
            ("start", timeframe.start_timestamp().to_string()),
            ("end", timeframe.end_timestamp().to_string()),
        ];
        self.client.get_with_query("stat/traffic", &params).await
    }

    /// Get DPI (Deep Packet Inspection) statistics.
    #[instrument(skip(self))]
    pub async fn get_dpi_stats(
        &self,
        timeframe: Option<Timeframe>,
    ) -> Result<Vec<serde_json::Value>> {
        if let Some(timeframe) = timeframe {
            let params = [
                ("start", timeframe.start_timestamp().to_string()),
                ("end", timeframe.end_timestamp().to_string()),
            ];
            self.client.get_with_query("stat/sitedpi", &params).await
        } else {
            self.client.get("stat/sitedpi").await
        }
    }
}
