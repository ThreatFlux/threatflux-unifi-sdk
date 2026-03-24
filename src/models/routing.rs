//! Routing models.
//!
//! # Example
//!
//! ```rust
//! use threatflux_unifi_sdk::models::routing::StaticRoute;
//!
//! // Create a static route via gateway
//! let route = StaticRoute::via_gateway("10.0.0.0/8", "192.168.1.1")
//!     .with_name("VPN Route")
//!     .with_distance(10);
//!
//! assert_eq!(route.network, "10.0.0.0/8");
//! assert_eq!(route.gateway, "192.168.1.1");
//! assert_eq!(route.distance, 10);
//!
//! // Create a blackhole route
//! let blackhole = StaticRoute::blackhole("192.168.100.0/24");
//! assert_eq!(blackhole.route_type, "blackhole");
//! ```

use serde::{Deserialize, Serialize};

/// Static route configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticRoute {
    /// Unique identifier.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Route name/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Whether the route is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Destination network (CIDR).
    #[serde(rename = "static-route_network")]
    pub network: String,

    /// Gateway/next-hop IP address.
    #[serde(rename = "static-route_nexthop")]
    pub gateway: String,

    /// Route type (nexthop-route, interface-route, blackhole).
    #[serde(rename = "type", default = "default_nexthop")]
    pub route_type: String,

    /// Interface (for interface-route type).
    #[serde(rename = "static-route_interface", skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,

    /// Route distance/metric.
    #[serde(rename = "static-route_distance", default = "default_distance")]
    pub distance: u32,

    /// Site ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
}

const fn default_true() -> bool {
    true
}

fn default_nexthop() -> String {
    "nexthop-route".to_string()
}

const fn default_distance() -> u32 {
    1
}

impl StaticRoute {
    /// Create a new static route via gateway.
    #[must_use]
    pub fn via_gateway(network: impl Into<String>, gateway: impl Into<String>) -> Self {
        Self {
            id: None,
            name: None,
            enabled: true,
            network: network.into(),
            gateway: gateway.into(),
            route_type: "nexthop-route".to_string(),
            interface: None,
            distance: 1,
            site_id: None,
        }
    }

    /// Create a blackhole route.
    #[must_use]
    pub fn blackhole(network: impl Into<String>) -> Self {
        Self {
            id: None,
            name: None,
            enabled: true,
            network: network.into(),
            gateway: String::new(),
            route_type: "blackhole".to_string(),
            interface: None,
            distance: 1,
            site_id: None,
        }
    }

    /// Create an interface route.
    #[must_use]
    pub fn via_interface(network: impl Into<String>, interface: impl Into<String>) -> Self {
        Self {
            id: None,
            name: None,
            enabled: true,
            network: network.into(),
            gateway: String::new(),
            route_type: "interface-route".to_string(),
            interface: Some(interface.into()),
            distance: 1,
            site_id: None,
        }
    }

    /// Set route name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set route distance/metric.
    #[must_use]
    pub const fn with_distance(mut self, distance: u32) -> Self {
        self.distance = distance;
        self
    }

    /// Enable or disable the route.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Routing table entry (read-only, from device).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteTableEntry {
    /// Destination network.
    #[serde(rename = "pfx")]
    pub prefix: String,

    /// Next hop.
    #[serde(rename = "nh", default)]
    pub next_hop: Vec<NextHop>,

    /// Route type (C=connected, S=static, O=OSPF, etc.).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub route_type: Option<String>,

    /// Administrative distance.
    #[serde(default)]
    pub distance: u32,

    /// Metric.
    #[serde(default)]
    pub metric: u32,

    /// Uptime.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime: Option<u64>,
}

/// Next hop information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextHop {
    /// Next hop IP.
    #[serde(rename = "ip", skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,

    /// Interface.
    #[serde(rename = "intf", skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,

    /// Weight (for ECMP).
    #[serde(default)]
    pub weight: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_route_via_gateway() {
        let route = StaticRoute::via_gateway("10.0.0.0/8", "192.168.1.1").with_name("VPN Route");
        assert_eq!(route.network, "10.0.0.0/8");
        assert_eq!(route.gateway, "192.168.1.1");
        assert_eq!(route.route_type, "nexthop-route");
        assert_eq!(route.name, Some("VPN Route".to_string()));
        assert!(route.enabled);
        assert_eq!(route.distance, 1);
        assert!(route.id.is_none());
        assert!(route.interface.is_none());
    }

    #[test]
    fn test_blackhole_route() {
        let route = StaticRoute::blackhole("192.168.100.0/24");
        assert_eq!(route.route_type, "blackhole");
        assert!(route.gateway.is_empty());
        assert_eq!(route.network, "192.168.100.0/24");
        assert!(route.enabled);
    }

    #[test]
    fn test_interface_route() {
        let route = StaticRoute::via_interface("172.16.0.0/12", "eth1");
        assert_eq!(route.route_type, "interface-route");
        assert_eq!(route.interface, Some("eth1".to_string()));
        assert!(route.gateway.is_empty());
    }

    #[test]
    fn test_route_with_distance() {
        let route = StaticRoute::via_gateway("0.0.0.0/0", "10.0.0.1").with_distance(10);
        assert_eq!(route.distance, 10);
    }

    #[test]
    fn test_route_with_enabled() {
        let route = StaticRoute::via_gateway("10.0.0.0/8", "192.168.1.1").with_enabled(false);
        assert!(!route.enabled);
    }

    #[test]
    fn test_route_serialization() {
        let route = StaticRoute::via_gateway("10.0.0.0/8", "192.168.1.1")
            .with_name("Test Route")
            .with_distance(5);

        let json = serde_json::to_string(&route).unwrap();
        assert!(json.contains("static-route_network"));
        assert!(json.contains("static-route_nexthop"));
        assert!(json.contains("10.0.0.0/8"));

        let deserialized: StaticRoute = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.network, "10.0.0.0/8");
        assert_eq!(deserialized.gateway, "192.168.1.1");
        assert_eq!(deserialized.distance, 5);
    }

    #[test]
    fn test_route_table_entry() {
        let entry = RouteTableEntry {
            prefix: "10.0.0.0/8".to_string(),
            next_hop: vec![NextHop {
                ip: Some("192.168.1.1".to_string()),
                interface: Some("eth0".to_string()),
                weight: 1,
            }],
            route_type: Some("S".to_string()),
            distance: 1,
            metric: 0,
            uptime: Some(3600),
        };

        assert_eq!(entry.prefix, "10.0.0.0/8");
        assert_eq!(entry.next_hop.len(), 1);
        assert_eq!(entry.next_hop[0].ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_next_hop() {
        let hop = NextHop { ip: Some("10.0.0.1".to_string()), interface: None, weight: 10 };

        assert_eq!(hop.ip, Some("10.0.0.1".to_string()));
        assert!(hop.interface.is_none());
        assert_eq!(hop.weight, 10);
    }

    #[test]
    fn test_route_table_entry_serialization() {
        let entry = RouteTableEntry {
            prefix: "0.0.0.0/0".to_string(),
            next_hop: vec![],
            route_type: Some("C".to_string()),
            distance: 0,
            metric: 0,
            uptime: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("pfx"));
        assert!(json.contains("0.0.0.0/0"));
    }
}
