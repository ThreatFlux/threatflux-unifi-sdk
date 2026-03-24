//! Common types used throughout the UniFi SDK.

use serde::{Deserialize, Serialize};

/// Network protocol for firewall rules, port forwarding, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    /// All protocols.
    #[default]
    All,
    /// TCP only.
    Tcp,
    /// UDP only.
    Udp,
    /// TCP and UDP.
    TcpUdp,
    /// ICMP.
    Icmp,
    /// ICMPv6.
    Icmpv6,
    /// GRE tunneling protocol.
    Gre,
    /// ESP (IPsec).
    Esp,
    /// AH (IPsec).
    Ah,
    /// SCTP.
    Sctp,
    /// Protocol by number.
    #[serde(untagged)]
    Number(u8),
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Tcp => write!(f, "tcp"),
            Self::Udp => write!(f, "udp"),
            Self::TcpUdp => write!(f, "tcp_udp"),
            Self::Icmp => write!(f, "icmp"),
            Self::Icmpv6 => write!(f, "icmpv6"),
            Self::Gre => write!(f, "gre"),
            Self::Esp => write!(f, "esp"),
            Self::Ah => write!(f, "ah"),
            Self::Sctp => write!(f, "sctp"),
            Self::Number(n) => write!(f, "{n}"),
        }
    }
}

/// Firewall action for rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FirewallAction {
    /// Accept the packet.
    #[default]
    Accept,
    /// Drop the packet silently.
    Drop,
    /// Reject the packet with ICMP response.
    Reject,
}

impl std::fmt::Display for FirewallAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accept => write!(f, "accept"),
            Self::Drop => write!(f, "drop"),
            Self::Reject => write!(f, "reject"),
        }
    }
}

/// Firewall ruleset types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FirewallRuleset {
    /// WAN inbound rules.
    WanIn,
    /// WAN outbound rules.
    WanOut,
    /// WAN local rules (to the router itself).
    WanLocal,
    /// LAN inbound rules.
    LanIn,
    /// LAN outbound rules.
    LanOut,
    /// LAN local rules.
    LanLocal,
    /// Guest inbound rules.
    GuestIn,
    /// Guest outbound rules.
    GuestOut,
    /// Guest local rules.
    GuestLocal,
    /// Inter-VLAN rules.
    InterVlan,
}

impl std::fmt::Display for FirewallRuleset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WanIn => write!(f, "WAN_IN"),
            Self::WanOut => write!(f, "WAN_OUT"),
            Self::WanLocal => write!(f, "WAN_LOCAL"),
            Self::LanIn => write!(f, "LAN_IN"),
            Self::LanOut => write!(f, "LAN_OUT"),
            Self::LanLocal => write!(f, "LAN_LOCAL"),
            Self::GuestIn => write!(f, "GUEST_IN"),
            Self::GuestOut => write!(f, "GUEST_OUT"),
            Self::GuestLocal => write!(f, "GUEST_LOCAL"),
            Self::InterVlan => write!(f, "INTER_VLAN"),
        }
    }
}

/// Firewall group types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FirewallGroupType {
    /// IPv4 address group.
    AddressGroup,
    /// IPv6 address group.
    Ipv6AddressGroup,
    /// Port group.
    PortGroup,
}

impl std::fmt::Display for FirewallGroupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddressGroup => write!(f, "address-group"),
            Self::Ipv6AddressGroup => write!(f, "ipv6-address-group"),
            Self::PortGroup => write!(f, "port-group"),
        }
    }
}

/// Network purpose types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkPurpose {
    /// Corporate network (standard LAN).
    #[default]
    Corporate,
    /// Guest network with isolation.
    Guest,
    /// WAN interface.
    Wan,
    /// VLAN-only (no DHCP/routing).
    VlanOnly,
    /// Remote user VPN network.
    RemoteUserVpn,
    /// Site-to-site VPN network.
    SiteVpn,
}

impl std::fmt::Display for NetworkPurpose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Corporate => write!(f, "corporate"),
            Self::Guest => write!(f, "guest"),
            Self::Wan => write!(f, "wan"),
            Self::VlanOnly => write!(f, "vlan-only"),
            Self::RemoteUserVpn => write!(f, "remote-user-vpn"),
            Self::SiteVpn => write!(f, "site-vpn"),
        }
    }
}

/// Device types in UniFi ecosystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    /// Unknown device type (default).
    #[default]
    Unknown,
    /// UniFi Dream Machine.
    Udm,
    /// UniFi Dream Machine Pro.
    UdmPro,
    /// UniFi Dream Machine SE.
    UdmSe,
    /// UniFi Dream Router.
    Udr,
    /// UniFi Switch.
    Usw,
    /// UniFi Access Point.
    Uap,
    /// UniFi Cloud Key.
    Uck,
    /// UniFi Security Gateway.
    Usg,
    /// UniFi Express.
    Uxg,
    /// Other/unknown device type.
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::Udm => write!(f, "udm"),
            Self::UdmPro => write!(f, "udmpro"),
            Self::UdmSe => write!(f, "udmse"),
            Self::Udr => write!(f, "udr"),
            Self::Usw => write!(f, "usw"),
            Self::Uap => write!(f, "uap"),
            Self::Uck => write!(f, "uck"),
            Self::Usg => write!(f, "usg"),
            Self::Uxg => write!(f, "uxg"),
            Self::Other(s) => write!(f, "{s}"),
        }
    }
}

/// Device state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeviceState {
    /// Device is online and connected.
    #[serde(rename = "1")]
    Connected,
    /// Device is offline/disconnected.
    #[serde(rename = "0")]
    #[default]
    Disconnected,
    /// Device is pending adoption.
    #[serde(rename = "2")]
    PendingAdoption,
    /// Device is upgrading firmware.
    #[serde(rename = "4")]
    Upgrading,
    /// Device is provisioning.
    #[serde(rename = "5")]
    Provisioning,
    /// Device heartbeat missed.
    #[serde(rename = "6")]
    HeartbeatMissed,
    /// Device is adopting.
    #[serde(rename = "7")]
    Adopting,
    /// Device adoption failed.
    #[serde(rename = "9")]
    AdoptionFailed,
    /// Device is isolated.
    #[serde(rename = "10")]
    Isolated,
}

/// `PoE` mode for switch ports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PoeMode {
    /// `PoE` disabled.
    #[default]
    Off,
    /// Auto `PoE` (802.3af/at).
    Auto,
    /// Passive 24V `PoE`.
    Passive24v,
    /// Passthrough `PoE`.
    Passthrough,
}

impl std::fmt::Display for PoeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "off"),
            Self::Auto => write!(f, "auto"),
            Self::Passive24v => write!(f, "passive24v"),
            Self::Passthrough => write!(f, "passthrough"),
        }
    }
}

/// Time frame for statistics queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Timeframe {
    /// Last hour.
    Hour,
    /// Last 24 hours.
    #[default]
    Day,
    /// Last 7 days.
    Week,
    /// Last 30 days.
    Month,
    /// Last 365 days.
    Year,
    /// Custom range (start timestamp, end timestamp).
    Custom(i64, i64),
}

impl Timeframe {
    /// Get the start timestamp for this timeframe.
    #[must_use]
    pub fn start_timestamp(&self) -> i64 {
        let now = chrono::Utc::now().timestamp();
        match self {
            Self::Hour => now - 3600,
            Self::Day => now - 86400,
            Self::Week => now - 604_800,
            Self::Month => now - 2_592_000,
            Self::Year => now - 31_536_000,
            Self::Custom(start, _) => *start,
        }
    }

    /// Get the end timestamp for this timeframe.
    #[must_use]
    pub fn end_timestamp(&self) -> i64 {
        match self {
            Self::Custom(_, end) => *end,
            _ => chrono::Utc::now().timestamp(),
        }
    }
}

/// Day of week for schedules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DayOfWeek {
    /// Monday.
    Mon,
    /// Tuesday.
    Tue,
    /// Wednesday.
    Wed,
    /// Thursday.
    Thu,
    /// Friday.
    Fri,
    /// Saturday.
    Sat,
    /// Sunday.
    Sun,
}

/// `IPsec` match type for firewall rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IpsecMatch {
    /// Match `IPsec` traffic.
    MatchIpsec,
    /// Match non-`IPsec` traffic.
    MatchNone,
}

/// DNS filtering level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DnsFilterLevel {
    /// No filtering.
    #[default]
    None,
    /// Work-appropriate filtering.
    Work,
    /// Family-safe filtering.
    Family,
}

impl std::fmt::Display for DnsFilterLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Work => write!(f, "work"),
            Self::Family => write!(f, "family"),
        }
    }
}

/// DNS record type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DnsRecordType {
    /// A record (IPv4).
    #[default]
    A,
    /// AAAA record (IPv6).
    Aaaa,
    /// CNAME record.
    Cname,
}

impl std::fmt::Display for DnsRecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A => write!(f, "A"),
            Self::Aaaa => write!(f, "AAAA"),
            Self::Cname => write!(f, "CNAME"),
        }
    }
}

/// Site-to-site VPN type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SiteVpnType {
    /// Auto-configured VPN.
    #[default]
    Auto,
    /// Manually configured VPN.
    Manual,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_display() {
        assert_eq!(Protocol::All.to_string(), "all");
        assert_eq!(Protocol::Tcp.to_string(), "tcp");
        assert_eq!(Protocol::TcpUdp.to_string(), "tcp_udp");
        assert_eq!(Protocol::Number(47).to_string(), "47");
    }

    #[test]
    fn test_firewall_action_display() {
        assert_eq!(FirewallAction::Accept.to_string(), "accept");
        assert_eq!(FirewallAction::Drop.to_string(), "drop");
        assert_eq!(FirewallAction::Reject.to_string(), "reject");
    }

    #[test]
    fn test_firewall_ruleset_display() {
        assert_eq!(FirewallRuleset::WanIn.to_string(), "WAN_IN");
        assert_eq!(FirewallRuleset::LanOut.to_string(), "LAN_OUT");
        assert_eq!(FirewallRuleset::InterVlan.to_string(), "INTER_VLAN");
    }

    #[test]
    fn test_timeframe_timestamps() {
        let day = Timeframe::Day;
        let now = chrono::Utc::now().timestamp();
        assert!(day.start_timestamp() < now);
        assert!(day.end_timestamp() <= now);
        assert!((day.end_timestamp() - day.start_timestamp() - 86400).abs() < 2);
    }

    #[test]
    fn test_protocol_serialize() {
        let tcp = Protocol::Tcp;
        let json = serde_json::to_string(&tcp).unwrap();
        assert_eq!(json, r#""tcp""#);

        let parsed: Protocol = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Protocol::Tcp);
    }
}
