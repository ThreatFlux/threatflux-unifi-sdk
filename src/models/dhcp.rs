//! DHCP reservation and lease models.
//!
//! # Example
//!
//! ```rust
//! use threatflux_unifi_sdk::models::dhcp::DhcpReservation;
//!
//! // Create a DHCP reservation
//! let reservation = DhcpReservation::new("aa:bb:cc:dd:ee:ff", "192.168.1.100")
//!     .with_name("Server")
//!     .with_network("default");
//!
//! assert_eq!(reservation.mac_address, "aa:bb:cc:dd:ee:ff");
//! assert_eq!(reservation.ip_address, "192.168.1.100");
//! assert_eq!(reservation.name, Some("Server".to_string()));
//! ```

use serde::{Deserialize, Serialize};

/// DHCP static reservation (fixed IP assignment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhcpReservation {
    /// Unique identifier.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// MAC address.
    #[serde(rename = "mac")]
    pub mac_address: String,

    /// Fixed IP address.
    #[serde(rename = "fixed_ip")]
    pub ip_address: String,

    /// Hostname/name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Network ID this reservation belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_id: Option<String>,

    /// Site ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
}

impl DhcpReservation {
    /// Create a new DHCP reservation.
    #[must_use]
    pub fn new(mac: impl Into<String>, ip: impl Into<String>) -> Self {
        Self {
            id: None,
            mac_address: mac.into(),
            ip_address: ip.into(),
            name: None,
            network_id: None,
            site_id: None,
        }
    }

    /// Set the hostname/name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the network ID.
    #[must_use]
    pub fn with_network(mut self, network_id: impl Into<String>) -> Self {
        self.network_id = Some(network_id.into());
        self
    }
}

/// DHCP lease information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhcpLease {
    /// MAC address.
    #[serde(rename = "mac")]
    pub mac_address: String,

    /// IP address.
    #[serde(rename = "ip")]
    pub ip_address: String,

    /// Hostname.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,

    /// Lease start time (Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,

    /// Lease end time (Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dhcp_reservation_new() {
        let res = DhcpReservation::new("aa:bb:cc:dd:ee:ff", "192.168.1.100");
        assert_eq!(res.mac_address, "aa:bb:cc:dd:ee:ff");
        assert_eq!(res.ip_address, "192.168.1.100");
        assert!(res.id.is_none());
        assert!(res.name.is_none());
    }

    #[test]
    fn test_dhcp_reservation_builder() {
        let res = DhcpReservation::new("aa:bb:cc:dd:ee:ff", "192.168.1.100")
            .with_name("Server")
            .with_network("LAN");

        assert_eq!(res.name, Some("Server".to_string()));
        assert_eq!(res.network_id, Some("LAN".to_string()));
    }

    #[test]
    fn test_dhcp_reservation_serialization() {
        let res = DhcpReservation::new("11:22:33:44:55:66", "10.0.0.50").with_name("Printer");

        let json = serde_json::to_string(&res).unwrap();
        assert!(json.contains("11:22:33:44:55:66"));
        assert!(json.contains("10.0.0.50"));
        assert!(json.contains("Printer"));

        let deserialized: DhcpReservation = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.mac_address, "11:22:33:44:55:66");
        assert_eq!(deserialized.ip_address, "10.0.0.50");
    }

    #[test]
    fn test_dhcp_lease() {
        let lease = DhcpLease {
            mac_address: "aa:bb:cc:dd:ee:ff".to_string(),
            ip_address: "192.168.1.50".to_string(),
            hostname: Some("laptop".to_string()),
            start: Some(1700000000),
            end: Some(1700086400),
        };

        assert_eq!(lease.mac_address, "aa:bb:cc:dd:ee:ff");
        assert_eq!(lease.ip_address, "192.168.1.50");
        assert_eq!(lease.hostname, Some("laptop".to_string()));
    }

    #[test]
    fn test_dhcp_lease_serialization() {
        let lease = DhcpLease {
            mac_address: "ff:ee:dd:cc:bb:aa".to_string(),
            ip_address: "192.168.1.100".to_string(),
            hostname: None,
            start: None,
            end: None,
        };

        let json = serde_json::to_string(&lease).unwrap();
        assert!(json.contains("ff:ee:dd:cc:bb:aa"));

        let deserialized: DhcpLease = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.mac_address, "ff:ee:dd:cc:bb:aa");
    }
}
