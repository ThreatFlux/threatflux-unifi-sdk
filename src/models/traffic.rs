//! Traffic rule models.
//!
//! # Example
//!
//! ```rust
//! use threatflux_unifi_sdk::models::traffic::TrafficRule;
//!
//! // Create a blocking rule for all clients
//! let rule = TrafficRule::block("Block social media")
//!     .for_all_clients()
//!     .blocking_categories(vec!["social".to_string()]);
//!
//! assert_eq!(rule.description, "Block social media");
//! assert_eq!(rule.action, "BLOCK");
//!
//! // Create a rate limiting rule
//! let limit = TrafficRule::rate_limit("Limit streaming", 10000, 5000)
//!     .for_network("net123");
//!
//! assert_eq!(limit.action, "RATE_LIMIT");
//! ```

use serde::{Deserialize, Serialize};

use crate::types::DayOfWeek;

/// Traffic management rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficRule {
    /// Unique identifier.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Rule name/description.
    pub description: String,

    /// Whether the rule is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Action (BLOCK, ALLOW, RATE_LIMIT).
    #[serde(default = "default_block")]
    pub action: String,

    /// Target type (CLIENT, NETWORK, INTERNET, etc.).
    #[serde(rename = "target_devices", default)]
    pub target_devices: Vec<TargetDevice>,

    /// Matching criteria.
    #[serde(default)]
    pub matching_target: String,

    /// Application categories to match.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub app_category_ids: Vec<String>,

    /// Specific applications to match.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub app_ids: Vec<String>,

    /// Domains to match.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domains: Vec<String>,

    /// IP addresses/CIDRs to match.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ip_addresses: Vec<String>,

    /// Regions to match.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<String>,

    /// Schedule - days of week.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schedule_days: Vec<DayOfWeek>,

    /// Schedule - start time (HH:MM).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_start: Option<String>,

    /// Schedule - end time (HH:MM).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_end: Option<String>,

    /// Bandwidth limit (for RATE_LIMIT action).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth_limit: Option<BandwidthLimit>,

    /// Site ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
}

const fn default_true() -> bool {
    true
}

fn default_block() -> String {
    "BLOCK".to_string()
}

/// Target device specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetDevice {
    /// Target type (CLIENT, NETWORK, ALL_CLIENTS).
    #[serde(rename = "type")]
    pub target_type: String,

    /// Client MAC address (for CLIENT type).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_mac: Option<String>,

    /// Network ID (for NETWORK type).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_id: Option<String>,
}

/// Bandwidth limit configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthLimit {
    /// Download limit in kbps.
    #[serde(rename = "download_limit_kbps", skip_serializing_if = "Option::is_none")]
    pub download_kbps: Option<u32>,

    /// Upload limit in kbps.
    #[serde(rename = "upload_limit_kbps", skip_serializing_if = "Option::is_none")]
    pub upload_kbps: Option<u32>,
}

impl TrafficRule {
    /// Create a new blocking rule.
    #[must_use]
    pub fn block(description: impl Into<String>) -> Self {
        Self {
            id: None,
            description: description.into(),
            enabled: true,
            action: "BLOCK".to_string(),
            target_devices: vec![],
            matching_target: String::new(),
            app_category_ids: vec![],
            app_ids: vec![],
            domains: vec![],
            ip_addresses: vec![],
            regions: vec![],
            schedule_days: vec![],
            schedule_start: None,
            schedule_end: None,
            bandwidth_limit: None,
            site_id: None,
        }
    }

    /// Create a rate limiting rule.
    #[must_use]
    pub fn rate_limit(
        description: impl Into<String>,
        download_kbps: u32,
        upload_kbps: u32,
    ) -> Self {
        Self {
            id: None,
            description: description.into(),
            enabled: true,
            action: "RATE_LIMIT".to_string(),
            target_devices: vec![],
            matching_target: String::new(),
            app_category_ids: vec![],
            app_ids: vec![],
            domains: vec![],
            ip_addresses: vec![],
            regions: vec![],
            schedule_days: vec![],
            schedule_start: None,
            schedule_end: None,
            bandwidth_limit: Some(BandwidthLimit {
                download_kbps: Some(download_kbps),
                upload_kbps: Some(upload_kbps),
            }),
            site_id: None,
        }
    }

    /// Create an allow rule.
    #[must_use]
    pub fn allow(description: impl Into<String>) -> Self {
        Self {
            id: None,
            description: description.into(),
            enabled: true,
            action: "ALLOW".to_string(),
            target_devices: vec![],
            matching_target: String::new(),
            app_category_ids: vec![],
            app_ids: vec![],
            domains: vec![],
            ip_addresses: vec![],
            regions: vec![],
            schedule_days: vec![],
            schedule_start: None,
            schedule_end: None,
            bandwidth_limit: None,
            site_id: None,
        }
    }

    /// Target all clients.
    #[must_use]
    pub fn for_all_clients(mut self) -> Self {
        self.target_devices.push(TargetDevice {
            target_type: "ALL_CLIENTS".to_string(),
            client_mac: None,
            network_id: None,
        });
        self
    }

    /// Target a specific client.
    #[must_use]
    pub fn for_client(mut self, mac: impl Into<String>) -> Self {
        self.target_devices.push(TargetDevice {
            target_type: "CLIENT".to_string(),
            client_mac: Some(mac.into()),
            network_id: None,
        });
        self
    }

    /// Target a network.
    #[must_use]
    pub fn for_network(mut self, network_id: impl Into<String>) -> Self {
        self.target_devices.push(TargetDevice {
            target_type: "NETWORK".to_string(),
            client_mac: None,
            network_id: Some(network_id.into()),
        });
        self
    }

    /// Block specific domains.
    #[must_use]
    pub fn blocking_domains(mut self, domains: Vec<String>) -> Self {
        self.domains = domains;
        self
    }

    /// Block specific IP addresses.
    #[must_use]
    pub fn blocking_ips(mut self, ips: Vec<String>) -> Self {
        self.ip_addresses = ips;
        self
    }

    /// Block specific app categories.
    #[must_use]
    pub fn blocking_categories(mut self, categories: Vec<String>) -> Self {
        self.app_category_ids = categories;
        self
    }

    /// Set schedule.
    #[must_use]
    pub fn with_schedule(mut self, days: Vec<DayOfWeek>, start: &str, end: &str) -> Self {
        self.schedule_days = days;
        self.schedule_start = Some(start.to_string());
        self.schedule_end = Some(end.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_rule() {
        let rule = TrafficRule::block("Block social media")
            .for_all_clients()
            .blocking_categories(vec!["social".to_string()]);

        assert_eq!(rule.description, "Block social media");
        assert_eq!(rule.action, "BLOCK");
        assert!(!rule.app_category_ids.is_empty());
        assert!(rule.enabled);
    }

    #[test]
    fn test_rate_limit_rule() {
        let rule = TrafficRule::rate_limit("Limit streaming", 10000, 5000).for_network("net123");

        assert_eq!(rule.action, "RATE_LIMIT");
        assert!(rule.bandwidth_limit.is_some());
        let limit = rule.bandwidth_limit.unwrap();
        assert_eq!(limit.download_kbps, Some(10000));
        assert_eq!(limit.upload_kbps, Some(5000));
    }

    #[test]
    fn test_block_rule_for_client() {
        let rule = TrafficRule::block("Block client").for_client("aa:bb:cc:dd:ee:ff");

        assert_eq!(rule.target_devices.len(), 1);
        assert_eq!(rule.target_devices[0].target_type, "CLIENT");
        assert_eq!(rule.target_devices[0].client_mac, Some("aa:bb:cc:dd:ee:ff".to_string()));
    }

    #[test]
    fn test_block_domains() {
        let rule = TrafficRule::block("Block sites")
            .blocking_domains(vec!["example.com".to_string(), "test.com".to_string()]);

        assert_eq!(rule.domains.len(), 2);
        assert!(rule.domains.contains(&"example.com".to_string()));
    }

    #[test]
    fn test_block_ips() {
        let rule = TrafficRule::block("Block IPs").blocking_ips(vec!["1.2.3.4".to_string()]);

        assert_eq!(rule.ip_addresses.len(), 1);
    }

    #[test]
    fn test_rule_with_schedule() {
        let rule = TrafficRule::block("Scheduled block").with_schedule(
            vec![DayOfWeek::Mon, DayOfWeek::Tue],
            "09:00",
            "17:00",
        );

        assert_eq!(rule.schedule_days.len(), 2);
        assert_eq!(rule.schedule_start, Some("09:00".to_string()));
        assert_eq!(rule.schedule_end, Some("17:00".to_string()));
    }

    #[test]
    fn test_target_device() {
        let target = TargetDevice {
            target_type: "CLIENT".to_string(),
            client_mac: Some("aa:bb:cc:dd:ee:ff".to_string()),
            network_id: None,
        };

        assert_eq!(target.target_type, "CLIENT");
    }

    #[test]
    fn test_bandwidth_limit() {
        let limit = BandwidthLimit { download_kbps: Some(50000), upload_kbps: Some(10000) };

        assert_eq!(limit.download_kbps, Some(50000));
        assert_eq!(limit.upload_kbps, Some(10000));
    }

    #[test]
    fn test_traffic_rule_serialization() {
        let rule = TrafficRule::block("Test rule").for_all_clients();

        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("Test rule"));
        assert!(json.contains("BLOCK"));
    }
}
