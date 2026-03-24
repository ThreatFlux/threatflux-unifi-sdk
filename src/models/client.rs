//! Client (connected device) models.

use serde::{Deserialize, Serialize};

/// A client/device connected to the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    /// Unique identifier.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Client MAC address.
    #[serde(rename = "mac")]
    pub mac_address: String,

    /// Client hostname.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,

    /// Client name (user-defined alias).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// IP address.
    #[serde(rename = "ip", skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// Whether the client is currently online.
    #[serde(rename = "is_wired", default)]
    pub is_wired: bool,

    /// Whether this client is blocked.
    #[serde(default)]
    pub blocked: bool,

    /// Whether this client is a guest.
    #[serde(default)]
    pub is_guest: bool,

    /// Network ID the client is connected to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_id: Option<String>,

    /// Associated access point MAC (for wireless).
    #[serde(rename = "ap_mac", skip_serializing_if = "Option::is_none")]
    pub ap_mac: Option<String>,

    /// ESSID (for wireless clients).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub essid: Option<String>,

    /// Radio type (ng, na, ac, ax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radio: Option<String>,

    /// Signal strength in dBm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<i32>,

    /// Noise level in dBm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise: Option<i32>,

    /// Transmit rate in Mbps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_rate: Option<u32>,

    /// Receive rate in Mbps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rx_rate: Option<u32>,

    /// Total bytes transmitted.
    #[serde(rename = "tx_bytes", default)]
    pub tx_bytes: u64,

    /// Total bytes received.
    #[serde(rename = "rx_bytes", default)]
    pub rx_bytes: u64,

    /// Uptime in seconds.
    #[serde(default)]
    pub uptime: u64,

    /// First seen timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<u64>,

    /// Last seen timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<u64>,

    /// OUI vendor name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oui: Option<String>,

    /// User ID if authenticated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Site ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,

    /// Fixed IP for this client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_ip: Option<String>,

    /// Use fixed IP.
    #[serde(default)]
    pub use_fixedip: bool,

    /// Note/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Client fingerprint device type.
    #[serde(rename = "dev_cat", skip_serializing_if = "Option::is_none")]
    pub device_category: Option<i32>,

    /// Device family.
    #[serde(rename = "dev_family", skip_serializing_if = "Option::is_none")]
    pub device_family: Option<i32>,

    /// Device vendor.
    #[serde(rename = "dev_vendor", skip_serializing_if = "Option::is_none")]
    pub device_vendor: Option<i32>,

    /// OS name.
    #[serde(rename = "os_name", skip_serializing_if = "Option::is_none")]
    pub os_name: Option<String>,
}

/// Client group configuration (QoS/user groups).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientGroup {
    /// Unique identifier.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Group name.
    pub name: String,

    /// Maximum download rate in Kbps (-1 for unlimited).
    #[serde(rename = "qos_rate_max_down", skip_serializing_if = "Option::is_none")]
    pub qos_rate_max_down: Option<i32>,

    /// Maximum upload rate in Kbps (-1 for unlimited).
    #[serde(rename = "qos_rate_max_up", skip_serializing_if = "Option::is_none")]
    pub qos_rate_max_up: Option<i32>,
}

impl ClientGroup {
    /// Create a new client group.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { id: None, name: name.into(), qos_rate_max_down: None, qos_rate_max_up: None }
    }

    /// Set QoS limits.
    #[must_use]
    pub const fn with_qos_limits(mut self, down_kbps: i32, up_kbps: i32) -> Self {
        self.qos_rate_max_down = Some(down_kbps);
        self.qos_rate_max_up = Some(up_kbps);
        self
    }
}

impl Client {
    /// Get display name (name, hostname, or MAC).
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.name.as_deref().or(self.hostname.as_deref()).unwrap_or(&self.mac_address)
    }

    /// Check if client is wireless.
    #[must_use]
    pub const fn is_wireless(&self) -> bool {
        !self.is_wired
    }
}

/// Client statistics for a specific time period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStats {
    /// MAC address.
    #[serde(rename = "mac")]
    pub mac_address: String,

    /// Bytes transmitted.
    #[serde(rename = "tx_bytes", default)]
    pub tx_bytes: u64,

    /// Bytes received.
    #[serde(rename = "rx_bytes", default)]
    pub rx_bytes: u64,

    /// Packets transmitted.
    #[serde(rename = "tx_packets", default)]
    pub tx_packets: u64,

    /// Packets received.
    #[serde(rename = "rx_packets", default)]
    pub rx_packets: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_display_name_with_name() {
        let client = Client {
            id: None,
            mac_address: "aa:bb:cc:dd:ee:ff".to_string(),
            hostname: Some("laptop".to_string()),
            name: Some("John's Laptop".to_string()),
            ip_address: None,
            is_wired: false,
            blocked: false,
            is_guest: false,
            network_id: None,
            ap_mac: None,
            essid: None,
            radio: None,
            signal: None,
            noise: None,
            tx_rate: None,
            rx_rate: None,
            tx_bytes: 0,
            rx_bytes: 0,
            uptime: 0,
            first_seen: None,
            last_seen: None,
            oui: None,
            user_id: None,
            site_id: None,
            fixed_ip: None,
            use_fixedip: false,
            note: None,
            device_category: None,
            device_family: None,
            device_vendor: None,
            os_name: None,
        };
        assert_eq!(client.display_name(), "John's Laptop");
    }

    #[test]
    fn test_client_is_wireless() {
        let mut client = Client {
            id: None,
            mac_address: "aa:bb:cc:dd:ee:ff".to_string(),
            hostname: None,
            name: None,
            ip_address: None,
            is_wired: false,
            blocked: false,
            is_guest: false,
            network_id: None,
            ap_mac: None,
            essid: None,
            radio: None,
            signal: None,
            noise: None,
            tx_rate: None,
            rx_rate: None,
            tx_bytes: 0,
            rx_bytes: 0,
            uptime: 0,
            first_seen: None,
            last_seen: None,
            oui: None,
            user_id: None,
            site_id: None,
            fixed_ip: None,
            use_fixedip: false,
            note: None,
            device_category: None,
            device_family: None,
            device_vendor: None,
            os_name: None,
        };
        assert!(client.is_wireless());
        client.is_wired = true;
        assert!(!client.is_wireless());
    }
}
