//! Network and VLAN models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::NetworkPurpose;

/// Network/VLAN configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    /// Unique identifier (read-only, assigned by controller).
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Network name.
    pub name: String,

    /// Network purpose (corporate, guest, wan, vlan-only).
    #[serde(default)]
    pub purpose: NetworkPurpose,

    /// Whether VLAN tagging is enabled.
    #[serde(default)]
    pub vlan_enabled: bool,

    /// VLAN ID (1-4094).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan: Option<u16>,

    /// Network subnet in CIDR notation (e.g., "192.168.1.0/24").
    #[serde(rename = "ip_subnet", skip_serializing_if = "Option::is_none")]
    pub subnet: Option<String>,

    /// Gateway IP address.
    #[serde(rename = "gateway_ip", skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,

    /// Whether DHCP server is enabled.
    #[serde(rename = "dhcpd_enabled", default)]
    pub dhcp_enabled: bool,

    /// DHCP range start.
    #[serde(rename = "dhcpd_start", skip_serializing_if = "Option::is_none")]
    pub dhcp_start: Option<String>,

    /// DHCP range stop.
    #[serde(rename = "dhcpd_stop", skip_serializing_if = "Option::is_none")]
    pub dhcp_stop: Option<String>,

    /// DHCP lease time in seconds.
    #[serde(rename = "dhcpd_leasetime", skip_serializing_if = "Option::is_none")]
    pub dhcp_lease: Option<u32>,

    /// Domain name for this network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_name: Option<String>,

    /// Whether IGMP snooping is enabled.
    #[serde(default)]
    pub igmp_snooping: bool,

    /// Network group (LAN, WAN, etc.).
    #[serde(default = "default_networkgroup")]
    pub networkgroup: String,

    /// Site ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,

    /// Whether this network is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// IPv6 settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_interface_type: Option<String>,

    /// DHCP DNS servers.
    #[serde(rename = "dhcpd_dns_enabled", default)]
    pub dhcp_dns_enabled: bool,

    /// Custom DNS servers for DHCP.
    #[serde(rename = "dhcpd_dns_1", skip_serializing_if = "Option::is_none")]
    pub dhcp_dns_1: Option<String>,

    /// Custom DNS servers for DHCP.
    #[serde(rename = "dhcpd_dns_2", skip_serializing_if = "Option::is_none")]
    pub dhcp_dns_2: Option<String>,

    /// DHCP gateway mode (auto or manual).
    #[serde(rename = "dhcpd_gateway_enabled", default)]
    pub dhcp_gateway_enabled: bool,

    /// Custom gateway for DHCP.
    #[serde(rename = "dhcpd_gateway", skip_serializing_if = "Option::is_none")]
    pub dhcp_gateway: Option<String>,

    /// Whether to advertise this network.
    #[serde(default)]
    pub is_nat: bool,

    /// DHCP relay enabled.
    #[serde(rename = "dhcp_relay_enabled", default)]
    pub dhcp_relay_enabled: bool,

    /// Auto-scale network.
    #[serde(rename = "auto_scale_enabled", default)]
    pub auto_scale_enabled: bool,

    /// Additional controller-specific fields (e.g., isolation settings).
    #[serde(flatten, default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_networkgroup() -> String {
    "LAN".to_string()
}

const fn default_true() -> bool {
    true
}

impl Network {
    /// Create a new corporate network.
    #[must_use]
    pub fn new_corporate(name: impl Into<String>, subnet: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            purpose: NetworkPurpose::Corporate,
            vlan_enabled: false,
            vlan: None,
            subnet: Some(subnet.into()),
            gateway: None,
            dhcp_enabled: true,
            dhcp_start: None,
            dhcp_stop: None,
            dhcp_lease: Some(86400),
            domain_name: None,
            igmp_snooping: false,
            networkgroup: "LAN".to_string(),
            site_id: None,
            enabled: true,
            ipv6_interface_type: None,
            dhcp_dns_enabled: false,
            dhcp_dns_1: None,
            dhcp_dns_2: None,
            dhcp_gateway_enabled: false,
            dhcp_gateway: None,
            is_nat: true,
            dhcp_relay_enabled: false,
            auto_scale_enabled: false,
            extra: HashMap::new(),
        }
    }

    /// Create a new VLAN network.
    #[must_use]
    pub fn new_vlan(name: impl Into<String>, vlan_id: u16, subnet: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            purpose: NetworkPurpose::Corporate,
            vlan_enabled: true,
            vlan: Some(vlan_id),
            subnet: Some(subnet.into()),
            gateway: None,
            dhcp_enabled: true,
            dhcp_start: None,
            dhcp_stop: None,
            dhcp_lease: Some(86400),
            domain_name: None,
            igmp_snooping: false,
            networkgroup: "LAN".to_string(),
            site_id: None,
            enabled: true,
            ipv6_interface_type: None,
            dhcp_dns_enabled: false,
            dhcp_dns_1: None,
            dhcp_dns_2: None,
            dhcp_gateway_enabled: false,
            dhcp_gateway: None,
            is_nat: true,
            dhcp_relay_enabled: false,
            auto_scale_enabled: false,
            extra: HashMap::new(),
        }
    }

    /// Create a new guest network.
    #[must_use]
    pub fn new_guest(name: impl Into<String>, vlan_id: u16, subnet: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            purpose: NetworkPurpose::Guest,
            vlan_enabled: true,
            vlan: Some(vlan_id),
            subnet: Some(subnet.into()),
            gateway: None,
            dhcp_enabled: true,
            dhcp_start: None,
            dhcp_stop: None,
            dhcp_lease: Some(86400),
            domain_name: None,
            igmp_snooping: false,
            networkgroup: "LAN".to_string(),
            site_id: None,
            enabled: true,
            ipv6_interface_type: None,
            dhcp_dns_enabled: false,
            dhcp_dns_1: None,
            dhcp_dns_2: None,
            dhcp_gateway_enabled: false,
            dhcp_gateway: None,
            is_nat: true,
            dhcp_relay_enabled: false,
            auto_scale_enabled: false,
            extra: HashMap::new(),
        }
    }

    /// Set the DHCP range.
    #[must_use]
    pub fn with_dhcp_range(mut self, start: impl Into<String>, stop: impl Into<String>) -> Self {
        self.dhcp_enabled = true;
        self.dhcp_start = Some(start.into());
        self.dhcp_stop = Some(stop.into());
        self
    }

    /// Set the DHCP lease time.
    #[must_use]
    pub const fn with_dhcp_lease(mut self, seconds: u32) -> Self {
        self.dhcp_lease = Some(seconds);
        self
    }

    /// Set the domain name.
    #[must_use]
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain_name = Some(domain.into());
        self
    }

    /// Set custom DNS servers for DHCP.
    #[must_use]
    pub fn with_dns_servers(mut self, dns1: impl Into<String>, dns2: Option<String>) -> Self {
        self.dhcp_dns_enabled = true;
        self.dhcp_dns_1 = Some(dns1.into());
        self.dhcp_dns_2 = dns2;
        self
    }

    /// Set a controller-specific field (e.g., isolation settings).
    #[must_use]
    pub fn with_custom_field(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }

    /// Enable IGMP snooping.
    #[must_use]
    pub const fn with_igmp_snooping(mut self, enabled: bool) -> Self {
        self.igmp_snooping = enabled;
        self
    }

    /// Set the gateway IP.
    #[must_use]
    pub fn with_gateway(mut self, gateway: impl Into<String>) -> Self {
        self.gateway = Some(gateway.into());
        self
    }
}

impl Default for Network {
    fn default() -> Self {
        Self::new_corporate("Default", "192.168.1.0/24")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_corporate_network() {
        let network = Network::new_corporate("Main LAN", "192.168.1.0/24");
        assert_eq!(network.name, "Main LAN");
        assert_eq!(network.purpose, NetworkPurpose::Corporate);
        assert!(!network.vlan_enabled);
        assert!(network.dhcp_enabled);
    }

    #[test]
    fn test_new_vlan_network() {
        let network = Network::new_vlan("IoT", 10, "192.168.10.0/24");
        assert_eq!(network.name, "IoT");
        assert!(network.vlan_enabled);
        assert_eq!(network.vlan, Some(10));
    }

    #[test]
    fn test_new_guest_network() {
        let network = Network::new_guest("Guest WiFi", 30, "192.168.30.0/24");
        assert_eq!(network.purpose, NetworkPurpose::Guest);
        assert!(network.vlan_enabled);
        assert_eq!(network.vlan, Some(30));
    }

    #[test]
    fn test_network_builder() {
        let network = Network::new_vlan("Servers", 20, "192.168.20.0/24")
            .with_dhcp_range("192.168.20.100", "192.168.20.200")
            .with_dhcp_lease(43200)
            .with_domain("servers.local")
            .with_dns_servers("1.1.1.1", Some("8.8.8.8".to_string()));

        assert_eq!(network.dhcp_start, Some("192.168.20.100".to_string()));
        assert_eq!(network.dhcp_stop, Some("192.168.20.200".to_string()));
        assert_eq!(network.dhcp_lease, Some(43200));
        assert_eq!(network.domain_name, Some("servers.local".to_string()));
        assert!(network.dhcp_dns_enabled);
    }

    #[test]
    fn test_network_serialization() {
        let network = Network::new_vlan("Test", 10, "192.168.10.0/24");
        let json = serde_json::to_string(&network).unwrap();
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"vlan_enabled\":true"));
        assert!(json.contains("\"vlan\":10"));
    }
}
