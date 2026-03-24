//! Port forwarding models.

use serde::{Deserialize, Serialize};

use crate::types::Protocol;

/// Port forwarding rule configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    /// Unique identifier (read-only).
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Rule name/description.
    pub name: String,

    /// Whether the rule is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Protocol (tcp, udp, tcp_udp).
    #[serde(rename = "proto", default = "default_tcp")]
    pub protocol: Protocol,

    /// Source IP or CIDR (empty = any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,

    /// Destination port or port range on WAN.
    #[serde(rename = "dst_port")]
    pub dst_port: String,

    /// Forward destination IP (internal host).
    #[serde(rename = "fwd")]
    pub forward_ip: String,

    /// Forward destination port (internal port).
    #[serde(rename = "fwd_port")]
    pub forward_port: String,

    /// Enable logging for this rule.
    #[serde(default)]
    pub log: bool,

    /// WAN interface (wan, wan2, or both).
    #[serde(rename = "pfwd_interface", default = "default_wan")]
    pub interface: String,

    /// Site ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
}

const fn default_true() -> bool {
    true
}

fn default_tcp() -> Protocol {
    Protocol::Tcp
}

fn default_wan() -> String {
    "wan".to_string()
}

impl PortForward {
    /// Create a new port forward rule.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        dst_port: impl Into<String>,
        forward_ip: impl Into<String>,
        forward_port: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            name: name.into(),
            enabled: true,
            protocol: Protocol::Tcp,
            src: None,
            dst_port: dst_port.into(),
            forward_ip: forward_ip.into(),
            forward_port: forward_port.into(),
            log: false,
            interface: "wan".to_string(),
            site_id: None,
        }
    }

    /// Create a TCP port forward.
    #[must_use]
    pub fn tcp(
        name: impl Into<String>,
        dst_port: impl Into<String>,
        forward_ip: impl Into<String>,
        forward_port: impl Into<String>,
    ) -> Self {
        Self::new(name, dst_port, forward_ip, forward_port).with_protocol(Protocol::Tcp)
    }

    /// Create a UDP port forward.
    #[must_use]
    pub fn udp(
        name: impl Into<String>,
        dst_port: impl Into<String>,
        forward_ip: impl Into<String>,
        forward_port: impl Into<String>,
    ) -> Self {
        Self::new(name, dst_port, forward_ip, forward_port).with_protocol(Protocol::Udp)
    }

    /// Create a TCP+UDP port forward.
    #[must_use]
    pub fn tcp_udp(
        name: impl Into<String>,
        dst_port: impl Into<String>,
        forward_ip: impl Into<String>,
        forward_port: impl Into<String>,
    ) -> Self {
        Self::new(name, dst_port, forward_ip, forward_port).with_protocol(Protocol::TcpUdp)
    }

    /// Set the protocol.
    #[must_use]
    pub const fn with_protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        self
    }

    /// Restrict source IP/CIDR.
    #[must_use]
    pub fn with_source(mut self, src: impl Into<String>) -> Self {
        self.src = Some(src.into());
        self
    }

    /// Enable logging.
    #[must_use]
    pub const fn with_logging(mut self, enabled: bool) -> Self {
        self.log = enabled;
        self
    }

    /// Set the WAN interface.
    #[must_use]
    pub fn with_interface(mut self, interface: impl Into<String>) -> Self {
        self.interface = interface.into();
        self
    }

    /// Use both WAN interfaces.
    #[must_use]
    pub fn with_both_wans(mut self) -> Self {
        self.interface = "both".to_string();
        self
    }
}

impl Default for PortForward {
    fn default() -> Self {
        Self::new("Default", "80", "192.168.1.100", "80")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_forward_new() {
        let pf = PortForward::new("Web Server", "80", "192.168.1.10", "8080");
        assert_eq!(pf.name, "Web Server");
        assert_eq!(pf.dst_port, "80");
        assert_eq!(pf.forward_ip, "192.168.1.10");
        assert_eq!(pf.forward_port, "8080");
        assert!(pf.enabled);
    }

    #[test]
    fn test_port_forward_tcp() {
        let pf = PortForward::tcp("SSH", "22", "192.168.1.5", "22");
        assert_eq!(pf.protocol, Protocol::Tcp);
    }

    #[test]
    fn test_port_forward_udp() {
        let pf = PortForward::udp("DNS", "53", "192.168.1.1", "53");
        assert_eq!(pf.protocol, Protocol::Udp);
    }

    #[test]
    fn test_port_forward_with_source() {
        let pf =
            PortForward::tcp("Restricted SSH", "22", "192.168.1.5", "22").with_source("10.0.0.0/8");
        assert_eq!(pf.src, Some("10.0.0.0/8".to_string()));
    }

    #[test]
    fn test_port_forward_serialization() {
        let pf = PortForward::tcp("Test", "443", "192.168.1.10", "443");
        let json = serde_json::to_string(&pf).unwrap();
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"dst_port\":\"443\""));
        assert!(json.contains("\"fwd\":\"192.168.1.10\""));
    }
}
