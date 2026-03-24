//! Site models.
//!
//! # Example
//!
//! ```rust
//! use threatflux_unifi_sdk::models::site::Site;
//!
//! let site = Site {
//!     id: Some("abc123".to_string()),
//!     name: "default".to_string(),
//!     description: Some("Home Network".to_string()),
//!     attr_hidden: false,
//!     attr_no_delete: false,
//!     role: Some("admin".to_string()),
//!     health: None,
//! };
//!
//! assert_eq!(site.display_name(), "Home Network");
//! assert!(site.is_default());
//! ```

use serde::{Deserialize, Serialize};

/// A UniFi site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    /// Site ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Site name (unique identifier in API paths).
    pub name: String,

    /// Site description/display name.
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether this is the default site.
    #[serde(default)]
    pub attr_hidden: bool,

    /// Whether site has pending changes.
    #[serde(default)]
    pub attr_no_delete: bool,

    /// Role of current user on this site.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Site health score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<Vec<SiteHealth>>,
}

impl Site {
    /// Get display name (description or name).
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.description.as_deref().unwrap_or(&self.name)
    }

    /// Check if this is the default site.
    #[must_use]
    pub fn is_default(&self) -> bool {
        self.name == "default"
    }
}

/// Site health information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteHealth {
    /// Subsystem name.
    pub subsystem: String,

    /// Health status.
    #[serde(default)]
    pub status: String,

    /// Number of items in this subsystem.
    #[serde(default)]
    pub num_user: u32,

    /// Number of guests.
    #[serde(default)]
    pub num_guest: u32,

    /// Number of IoT devices.
    #[serde(default)]
    pub num_iot: u32,

    /// Number of adopted devices.
    #[serde(rename = "num_adopted", default)]
    pub num_adopted: u32,

    /// Number of pending devices.
    #[serde(rename = "num_pending", default)]
    pub num_pending: u32,

    /// WAN IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wan_ip: Option<String>,

    /// ISP name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isp_name: Option<String>,

    /// Download speed in bps.
    #[serde(rename = "rx_bytes-r", default)]
    pub rx_bytes_rate: u64,

    /// Upload speed in bps.
    #[serde(rename = "tx_bytes-r", default)]
    pub tx_bytes_rate: u64,
}

/// Site statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteStats {
    /// Site ID.
    #[serde(rename = "site_id")]
    pub site_id: String,

    /// Total bytes received.
    #[serde(rename = "rx_bytes", default)]
    pub rx_bytes: u64,

    /// Total bytes transmitted.
    #[serde(rename = "tx_bytes", default)]
    pub tx_bytes: u64,

    /// Number of active clients.
    #[serde(rename = "num_sta", default)]
    pub num_clients: u32,

    /// Number of users.
    #[serde(rename = "num_user", default)]
    pub num_users: u32,

    /// Number of guests.
    #[serde(rename = "num_guest", default)]
    pub num_guests: u32,

    /// WAN download rate.
    #[serde(rename = "wan-rx_bytes", default)]
    pub wan_rx_bytes: u64,

    /// WAN upload rate.
    #[serde(rename = "wan-tx_bytes", default)]
    pub wan_tx_bytes: u64,

    /// LAN download rate.
    #[serde(rename = "lan-rx_bytes", default)]
    pub lan_rx_bytes: u64,

    /// LAN upload rate.
    #[serde(rename = "lan-tx_bytes", default)]
    pub lan_tx_bytes: u64,
}

/// System information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Controller version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Build number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,

    /// Controller name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Hostname.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,

    /// Is cloud key.
    #[serde(default)]
    pub is_cloudkey: bool,

    /// Is UniFi OS.
    #[serde(default)]
    pub ubnt_device_type: Option<String>,

    /// Uptime in seconds.
    #[serde(default)]
    pub uptime: u64,

    /// Timezone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_site_display_name() {
        let site = Site {
            id: Some("abc".to_string()),
            name: "default".to_string(),
            description: Some("Home Network".to_string()),
            attr_hidden: false,
            attr_no_delete: false,
            role: None,
            health: None,
        };
        assert_eq!(site.display_name(), "Home Network");
    }

    #[test]
    fn test_site_display_name_fallback() {
        let site = Site {
            id: None,
            name: "mysite".to_string(),
            description: None,
            attr_hidden: false,
            attr_no_delete: false,
            role: None,
            health: None,
        };
        assert_eq!(site.display_name(), "mysite");
    }

    #[test]
    fn test_site_is_default() {
        let site = Site {
            id: None,
            name: "default".to_string(),
            description: None,
            attr_hidden: false,
            attr_no_delete: false,
            role: None,
            health: None,
        };
        assert!(site.is_default());
    }

    #[test]
    fn test_site_is_not_default() {
        let site = Site {
            id: None,
            name: "office".to_string(),
            description: None,
            attr_hidden: false,
            attr_no_delete: false,
            role: None,
            health: None,
        };
        assert!(!site.is_default());
    }

    #[test]
    fn test_site_health() {
        let health = SiteHealth {
            subsystem: "wlan".to_string(),
            status: "ok".to_string(),
            num_user: 10,
            num_guest: 5,
            num_iot: 3,
            num_adopted: 4,
            num_pending: 0,
            wan_ip: Some("1.2.3.4".to_string()),
            isp_name: Some("Comcast".to_string()),
            rx_bytes_rate: 1000000,
            tx_bytes_rate: 500000,
        };

        assert_eq!(health.subsystem, "wlan");
        assert_eq!(health.num_user, 10);
        assert_eq!(health.wan_ip, Some("1.2.3.4".to_string()));
    }

    #[test]
    fn test_site_stats() {
        let stats = SiteStats {
            site_id: "abc123".to_string(),
            rx_bytes: 1000000000,
            tx_bytes: 500000000,
            num_clients: 25,
            num_users: 20,
            num_guests: 5,
            wan_rx_bytes: 900000000,
            wan_tx_bytes: 400000000,
            lan_rx_bytes: 100000000,
            lan_tx_bytes: 100000000,
        };

        assert_eq!(stats.site_id, "abc123");
        assert_eq!(stats.num_clients, 25);
    }

    #[test]
    fn test_system_info() {
        let info = SystemInfo {
            version: Some("7.5.187".to_string()),
            build: Some("abc123".to_string()),
            name: Some("UniFi".to_string()),
            hostname: Some("unifi.local".to_string()),
            is_cloudkey: false,
            ubnt_device_type: Some("UDM-Pro".to_string()),
            uptime: 86400,
            timezone: Some("America/New_York".to_string()),
        };

        assert_eq!(info.version, Some("7.5.187".to_string()));
        assert_eq!(info.uptime, 86400);
    }

    #[test]
    fn test_site_serialization() {
        let site = Site {
            id: Some("abc".to_string()),
            name: "default".to_string(),
            description: Some("Test".to_string()),
            attr_hidden: false,
            attr_no_delete: true,
            role: Some("admin".to_string()),
            health: None,
        };

        let json = serde_json::to_string(&site).unwrap();
        assert!(json.contains("default"));
        assert!(json.contains("admin"));
    }
}
