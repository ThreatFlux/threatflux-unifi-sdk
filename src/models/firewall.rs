//! Firewall rule and group models.

use serde::{Deserialize, Serialize};

use crate::types::{FirewallAction, FirewallGroupType, FirewallRuleset, IpsecMatch, Protocol};

/// Firewall rule configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    /// Unique identifier (read-only).
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Rule name/description.
    pub name: String,

    /// Whether the rule is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Rule index for ordering (lower = higher priority).
    #[serde(default)]
    pub rule_index: i32,

    /// Ruleset this rule belongs to.
    pub ruleset: FirewallRuleset,

    /// Action to take (accept, drop, reject).
    #[serde(default)]
    pub action: FirewallAction,

    /// Protocol to match.
    #[serde(rename = "protocol", default)]
    pub protocol: Protocol,

    /// Source firewall group IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub src_firewallgroup_ids: Vec<String>,

    /// Source IP address or CIDR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_address: Option<String>,

    /// Source network ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_networkconf_id: Option<String>,

    /// Source network type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_networkconf_type: Option<String>,

    /// Source MAC address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_mac_address: Option<String>,

    /// Source port (for TCP/UDP).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_port: Option<String>,

    /// Destination firewall group IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dst_firewallgroup_ids: Vec<String>,

    /// Destination IP address or CIDR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_address: Option<String>,

    /// Destination network ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_networkconf_id: Option<String>,

    /// Destination network type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_networkconf_type: Option<String>,

    /// Destination port or port range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_port: Option<String>,

    /// Enable logging for this rule.
    #[serde(default)]
    pub logging: bool,

    /// Match new connections.
    #[serde(default)]
    pub state_new: bool,

    /// Match established connections.
    #[serde(default)]
    pub state_established: bool,

    /// Match invalid packets.
    #[serde(default)]
    pub state_invalid: bool,

    /// Match related connections.
    #[serde(default)]
    pub state_related: bool,

    /// IPsec matching.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipsec: Option<IpsecMatch>,

    /// Site ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,

    /// ICMP type name (for ICMP protocol).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icmp_typename: Option<String>,
}

const fn default_true() -> bool {
    true
}

impl FirewallRule {
    /// Create a new firewall rule.
    #[must_use]
    pub fn new(name: impl Into<String>, ruleset: FirewallRuleset, action: FirewallAction) -> Self {
        Self {
            id: None,
            name: name.into(),
            enabled: true,
            rule_index: 2000,
            ruleset,
            action,
            protocol: Protocol::All,
            src_firewallgroup_ids: Vec::new(),
            src_address: None,
            src_networkconf_id: None,
            src_networkconf_type: None,
            src_mac_address: None,
            src_port: None,
            dst_firewallgroup_ids: Vec::new(),
            dst_address: None,
            dst_networkconf_id: None,
            dst_networkconf_type: None,
            dst_port: None,
            logging: false,
            state_new: false,
            state_established: false,
            state_invalid: false,
            state_related: false,
            ipsec: None,
            site_id: None,
            icmp_typename: None,
        }
    }

    /// Create an accept rule.
    #[must_use]
    pub fn accept(name: impl Into<String>, ruleset: FirewallRuleset) -> Self {
        Self::new(name, ruleset, FirewallAction::Accept)
    }

    /// Create a drop rule.
    #[must_use]
    pub fn drop(name: impl Into<String>, ruleset: FirewallRuleset) -> Self {
        Self::new(name, ruleset, FirewallAction::Drop)
    }

    /// Create a reject rule.
    #[must_use]
    pub fn reject(name: impl Into<String>, ruleset: FirewallRuleset) -> Self {
        Self::new(name, ruleset, FirewallAction::Reject)
    }

    /// Set the rule index (priority).
    #[must_use]
    pub const fn with_index(mut self, index: i32) -> Self {
        self.rule_index = index;
        self
    }

    /// Set the protocol.
    #[must_use]
    pub const fn with_protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        self
    }

    /// Set the source address.
    #[must_use]
    pub fn with_src_address(mut self, address: impl Into<String>) -> Self {
        self.src_address = Some(address.into());
        self
    }

    /// Set the source network.
    #[must_use]
    pub fn with_src_network(mut self, network_id: impl Into<String>) -> Self {
        self.src_networkconf_id = Some(network_id.into());
        self.src_networkconf_type = Some("NETv4".to_string());
        self
    }

    /// Add a source firewall group.
    #[must_use]
    pub fn with_src_group(mut self, group_id: impl Into<String>) -> Self {
        self.src_firewallgroup_ids.push(group_id.into());
        self
    }

    /// Set the destination address.
    #[must_use]
    pub fn with_dst_address(mut self, address: impl Into<String>) -> Self {
        self.dst_address = Some(address.into());
        self
    }

    /// Set the destination network.
    #[must_use]
    pub fn with_dst_network(mut self, network_id: impl Into<String>) -> Self {
        self.dst_networkconf_id = Some(network_id.into());
        self.dst_networkconf_type = Some("NETv4".to_string());
        self
    }

    /// Add a destination firewall group.
    #[must_use]
    pub fn with_dst_group(mut self, group_id: impl Into<String>) -> Self {
        self.dst_firewallgroup_ids.push(group_id.into());
        self
    }

    /// Set the destination port(s).
    #[must_use]
    pub fn with_dst_port(mut self, port: impl Into<String>) -> Self {
        self.dst_port = Some(port.into());
        self
    }

    /// Enable logging.
    #[must_use]
    pub const fn with_logging(mut self, enabled: bool) -> Self {
        self.logging = enabled;
        self
    }

    /// Match established and related connections.
    #[must_use]
    pub const fn with_state_established_related(mut self) -> Self {
        self.state_established = true;
        self.state_related = true;
        self
    }

    /// Match invalid packets.
    #[must_use]
    pub const fn with_state_invalid(mut self) -> Self {
        self.state_invalid = true;
        self
    }

    /// Match new connections.
    #[must_use]
    pub const fn with_state_new(mut self) -> Self {
        self.state_new = true;
        self
    }
}

/// Firewall group (address group or port group).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallGroup {
    /// Unique identifier (read-only).
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Group name.
    pub name: String,

    /// Group type (address-group, port-group, ipv6-address-group).
    #[serde(rename = "group_type")]
    pub group_type: FirewallGroupType,

    /// Group members (addresses or ports).
    #[serde(default)]
    pub group_members: Vec<String>,

    /// Site ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
}

impl FirewallGroup {
    /// Create a new address group.
    #[must_use]
    pub fn address_group(name: impl Into<String>, members: Vec<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            group_type: FirewallGroupType::AddressGroup,
            group_members: members,
            site_id: None,
        }
    }

    /// Create a new IPv6 address group.
    #[must_use]
    pub fn ipv6_address_group(name: impl Into<String>, members: Vec<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            group_type: FirewallGroupType::Ipv6AddressGroup,
            group_members: members,
            site_id: None,
        }
    }

    /// Create a new port group.
    #[must_use]
    pub fn port_group(name: impl Into<String>, ports: Vec<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            group_type: FirewallGroupType::PortGroup,
            group_members: ports,
            site_id: None,
        }
    }

    /// Add a member to the group.
    #[must_use]
    pub fn with_member(mut self, member: impl Into<String>) -> Self {
        self.group_members.push(member.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firewall_rule_builder() {
        let rule = FirewallRule::drop("Block Bad IPs", FirewallRuleset::WanIn)
            .with_index(2001)
            .with_src_group("bad-actors")
            .with_logging(true);

        assert_eq!(rule.name, "Block Bad IPs");
        assert_eq!(rule.action, FirewallAction::Drop);
        assert_eq!(rule.ruleset, FirewallRuleset::WanIn);
        assert_eq!(rule.rule_index, 2001);
        assert!(rule.logging);
        assert_eq!(rule.src_firewallgroup_ids, vec!["bad-actors"]);
    }

    #[test]
    fn test_firewall_rule_state_matching() {
        let rule = FirewallRule::accept("Allow Established", FirewallRuleset::WanIn)
            .with_state_established_related();

        assert!(rule.state_established);
        assert!(rule.state_related);
        assert!(!rule.state_new);
        assert!(!rule.state_invalid);
    }

    #[test]
    fn test_address_group() {
        let group = FirewallGroup::address_group(
            "RFC1918",
            vec![
                "10.0.0.0/8".to_string(),
                "172.16.0.0/12".to_string(),
                "192.168.0.0/16".to_string(),
            ],
        );

        assert_eq!(group.name, "RFC1918");
        assert_eq!(group.group_type, FirewallGroupType::AddressGroup);
        assert_eq!(group.group_members.len(), 3);
    }

    #[test]
    fn test_port_group() {
        let group =
            FirewallGroup::port_group("Web Ports", vec!["80".to_string(), "443".to_string()]);

        assert_eq!(group.group_type, FirewallGroupType::PortGroup);
        assert_eq!(group.group_members, vec!["80", "443"]);
    }

    #[test]
    fn test_rule_serialization() {
        let rule = FirewallRule::accept("Test", FirewallRuleset::LanIn)
            .with_protocol(Protocol::Tcp)
            .with_dst_port("443");

        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"action\":\"accept\""));
    }
}
