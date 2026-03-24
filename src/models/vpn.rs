//! VPN configuration models.
//!
//! # Example
//!
//! ```rust
//! use threatflux_unifi_sdk::models::vpn::{WireGuardServer, WireGuardPeer, SiteVpn};
//!
//! // Create a WireGuard server
//! let server = WireGuardServer::new("Home VPN", "10.10.0.1/24")
//!     .with_port(51821)
//!     .with_dns(vec!["1.1.1.1".to_string()]);
//!
//! assert_eq!(server.name, "Home VPN");
//! assert_eq!(server.port, 51821);
//!
//! // Create a WireGuard peer
//! let peer = WireGuardPeer::new("Phone", "abc123pubkey")
//!     .with_allowed_ips(vec!["10.10.0.2/32".to_string()]);
//!
//! assert_eq!(peer.name, "Phone");
//! ```

use serde::{Deserialize, Serialize};

use crate::types::SiteVpnType;

/// WireGuard VPN server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardServer {
    /// Unique identifier.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Server name.
    pub name: String,

    /// Whether enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Interface address (CIDR).
    #[serde(rename = "interface_address")]
    pub address: String,

    /// Listen port.
    #[serde(default = "default_wg_port")]
    pub port: u16,

    /// Server private key (generated if not provided).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,

    /// Server public key (derived from private key).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,

    /// DNS servers for clients.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dns_servers: Vec<String>,

    /// Allowed networks (routes pushed to clients).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_networks: Vec<String>,

    /// Site ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
}

const fn default_true() -> bool {
    true
}

const fn default_wg_port() -> u16 {
    51820
}

impl WireGuardServer {
    /// Create a new WireGuard server.
    #[must_use]
    pub fn new(name: impl Into<String>, address: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            enabled: true,
            address: address.into(),
            port: 51820,
            private_key: None,
            public_key: None,
            dns_servers: vec![],
            allowed_networks: vec![],
            site_id: None,
        }
    }

    /// Set the listen port.
    #[must_use]
    pub const fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set DNS servers.
    #[must_use]
    pub fn with_dns(mut self, dns: Vec<String>) -> Self {
        self.dns_servers = dns;
        self
    }

    /// Set allowed networks.
    #[must_use]
    pub fn with_allowed_networks(mut self, networks: Vec<String>) -> Self {
        self.allowed_networks = networks;
        self
    }
}

/// WireGuard VPN peer/client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardPeer {
    /// Unique identifier.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Peer name.
    pub name: String,

    /// Whether enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Peer public key.
    pub public_key: String,

    /// Preshared key (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preshared_key: Option<String>,

    /// Allowed IPs for this peer.
    #[serde(default)]
    pub allowed_ips: Vec<String>,

    /// Persistent keepalive interval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistent_keepalive: Option<u16>,

    /// Server ID this peer belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,

    /// Site ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
}

impl WireGuardPeer {
    /// Create a new WireGuard peer.
    #[must_use]
    pub fn new(name: impl Into<String>, public_key: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            enabled: true,
            public_key: public_key.into(),
            preshared_key: None,
            allowed_ips: vec![],
            persistent_keepalive: Some(25),
            server_id: None,
            site_id: None,
        }
    }

    /// Set allowed IPs.
    #[must_use]
    pub fn with_allowed_ips(mut self, ips: Vec<String>) -> Self {
        self.allowed_ips = ips;
        self
    }

    /// Set preshared key.
    #[must_use]
    pub fn with_preshared_key(mut self, key: impl Into<String>) -> Self {
        self.preshared_key = Some(key.into());
        self
    }
}

/// Site-to-site VPN configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteVpn {
    /// Unique identifier.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// VPN name.
    pub name: String,

    /// Whether enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// VPN type.
    #[serde(rename = "vpn_type", default)]
    pub vpn_type: SiteVpnType,

    /// Remote host/IP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_host: Option<String>,

    /// Remote subnets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remote_subnets: Vec<String>,

    /// Local subnets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_subnets: Vec<String>,

    /// Pre-shared key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psk: Option<String>,

    /// IKE version (1 or 2).
    #[serde(default = "default_ike_version")]
    pub ike_version: u8,

    /// Site ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
}

const fn default_ike_version() -> u8 {
    2
}

impl SiteVpn {
    /// Create a new site-to-site VPN.
    #[must_use]
    pub fn new(name: impl Into<String>, remote_host: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            enabled: true,
            vpn_type: SiteVpnType::Auto,
            remote_host: Some(remote_host.into()),
            remote_subnets: vec![],
            local_subnets: vec![],
            psk: None,
            ike_version: 2,
            site_id: None,
        }
    }

    /// Set remote subnets.
    #[must_use]
    pub fn with_remote_subnets(mut self, subnets: Vec<String>) -> Self {
        self.remote_subnets = subnets;
        self
    }

    /// Set local subnets.
    #[must_use]
    pub fn with_local_subnets(mut self, subnets: Vec<String>) -> Self {
        self.local_subnets = subnets;
        self
    }

    /// Set pre-shared key.
    #[must_use]
    pub fn with_psk(mut self, psk: impl Into<String>) -> Self {
        self.psk = Some(psk.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wireguard_server() {
        let server = WireGuardServer::new("Home VPN", "10.10.0.1/24")
            .with_port(51821)
            .with_dns(vec!["1.1.1.1".to_string()]);

        assert_eq!(server.name, "Home VPN");
        assert_eq!(server.port, 51821);
        assert_eq!(server.dns_servers, vec!["1.1.1.1"]);
        assert!(server.enabled);
        assert!(server.id.is_none());
    }

    #[test]
    fn test_wireguard_server_with_networks() {
        let server = WireGuardServer::new("VPN", "10.0.0.1/24")
            .with_allowed_networks(vec!["192.168.1.0/24".to_string(), "10.0.0.0/8".to_string()]);

        assert_eq!(server.allowed_networks.len(), 2);
    }

    #[test]
    fn test_wireguard_peer() {
        let peer = WireGuardPeer::new("Phone", "abc123pubkey")
            .with_allowed_ips(vec!["10.10.0.2/32".to_string()]);

        assert_eq!(peer.name, "Phone");
        assert_eq!(peer.public_key, "abc123pubkey");
        assert!(peer.enabled);
        assert_eq!(peer.persistent_keepalive, Some(25));
    }

    #[test]
    fn test_wireguard_peer_with_psk() {
        let peer = WireGuardPeer::new("Laptop", "xyz789pubkey").with_preshared_key("secretpsk");

        assert_eq!(peer.preshared_key, Some("secretpsk".to_string()));
    }

    #[test]
    fn test_site_vpn() {
        let vpn = SiteVpn::new("Office Link", "vpn.office.com")
            .with_remote_subnets(vec!["192.168.100.0/24".to_string()])
            .with_psk("supersecret");

        assert_eq!(vpn.name, "Office Link");
        assert!(vpn.psk.is_some());
        assert_eq!(vpn.ike_version, 2);
        assert!(vpn.enabled);
    }

    #[test]
    fn test_site_vpn_with_local_subnets() {
        let vpn = SiteVpn::new("Branch", "branch.example.com")
            .with_local_subnets(vec!["10.0.0.0/24".to_string()])
            .with_remote_subnets(vec!["10.1.0.0/24".to_string()]);

        assert_eq!(vpn.local_subnets.len(), 1);
        assert_eq!(vpn.remote_subnets.len(), 1);
    }

    #[test]
    fn test_wireguard_server_serialization() {
        let server = WireGuardServer::new("Test", "10.0.0.1/24");
        let json = serde_json::to_string(&server).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("10.0.0.1/24"));
    }

    #[test]
    fn test_wireguard_peer_serialization() {
        let peer = WireGuardPeer::new("Test Peer", "pubkey123");
        let json = serde_json::to_string(&peer).unwrap();
        assert!(json.contains("Test Peer"));
        assert!(json.contains("pubkey123"));
    }

    #[test]
    fn test_site_vpn_serialization() {
        let vpn = SiteVpn::new("Test VPN", "remote.host");
        let json = serde_json::to_string(&vpn).unwrap();
        assert!(json.contains("Test VPN"));
        assert!(json.contains("remote.host"));
    }
}
