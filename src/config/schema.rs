//! Declarative configuration schema for UniFi automation.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::{
    ClientGroup, DhcpReservation, DnsRecord, FirewallGroup, FirewallRule, Network, PortForward,
    SiteVpn, TrafficRule, WireGuardPeer, WireGuardServer,
};
use crate::types::{
    DnsFilterLevel, FirewallAction, FirewallGroupType, FirewallRuleset, NetworkPurpose, Protocol,
    SiteVpnType,
};

fn default_site() -> String {
    "default".to_string()
}

fn default_timeout() -> u64 {
    30
}

/// Connection configuration for the UniFi controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiConnectionConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    #[serde(default = "default_site")]
    pub site: String,
    #[serde(default)]
    pub verify_ssl: bool,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

impl UnifiConnectionConfig {
    /// Convert to SDK client config.
    #[must_use]
    pub fn to_client_config(&self) -> crate::client::UnifiConfig {
        crate::client::UnifiConfig::new(&self.host, &self.username, &self.password)
            .with_site(self.site.clone())
            .with_verify_ssl(self.verify_ssl)
            .with_timeout(self.timeout_secs)
    }
}

/// Top-level declarative configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiDeclarativeConfig {
    pub unifi: UnifiConnectionConfig,

    #[serde(default)]
    pub networks: Vec<NetworkConfig>,

    #[serde(default)]
    pub firewall_groups: Vec<FirewallGroupConfig>,

    #[serde(default)]
    pub firewall: FirewallConfig,

    #[serde(default)]
    pub port_forward: Vec<PortForwardConfig>,

    #[serde(default)]
    pub traffic_rules: Vec<TrafficRuleConfig>,

    #[serde(default)]
    pub vpn: Option<VpnConfig>,

    #[serde(default)]
    pub dhcp_reservations: Vec<DhcpReservationConfig>,

    #[serde(default)]
    pub dns: Option<DnsConfig>,

    #[serde(default)]
    pub clients: Option<ClientsConfig>,
}

/// Network/VLAN configuration in declarative format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,

    #[serde(default)]
    pub purpose: NetworkPurpose,

    #[serde(default)]
    pub vlan_enabled: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vlan: Option<u16>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subnet: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dhcp: Option<DhcpConfig>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    #[serde(default)]
    pub igmp_snooping: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub networkgroup: Option<String>,

    #[serde(flatten, default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}

impl NetworkConfig {
    /// Convert to SDK network model.
    #[must_use]
    pub fn to_model(&self) -> Network {
        let mut network =
            Network::new_corporate(self.name.clone(), self.subnet.clone().unwrap_or_default());
        network.name.clone_from(&self.name);
        network.purpose = self.purpose;
        network.vlan_enabled = if self.vlan_enabled { true } else { self.vlan.is_some() };
        network.vlan = self.vlan;
        network.subnet.clone_from(&self.subnet);
        network.gateway.clone_from(&self.gateway);
        network.domain_name.clone_from(&self.domain);
        network.igmp_snooping = self.igmp_snooping;
        if let Some(group) = &self.networkgroup {
            network.networkgroup.clone_from(group);
        }

        if let Some(dhcp) = &self.dhcp {
            network.dhcp_enabled = dhcp.enabled;
            network.dhcp_start.clone_from(&dhcp.start);
            network.dhcp_stop.clone_from(&dhcp.stop);
            network.dhcp_lease = dhcp.lease;
            if let Some(dns) = &dhcp.dns {
                network.dhcp_dns_enabled = !dns.is_empty();
                network.dhcp_dns_1 = dns.first().cloned();
                network.dhcp_dns_2 = dns.get(1).cloned();
            }
        }

        for (key, value) in &self.extra {
            network.extra.insert(key.clone(), value.clone());
        }

        network
    }
}

/// DHCP settings for a network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhcpConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lease: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dns: Option<Vec<String>>,
}

/// Firewall group config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallGroupConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: FirewallGroupType,
    #[serde(default)]
    pub members: Vec<String>,
}

impl FirewallGroupConfig {
    /// Convert to SDK firewall group model.
    #[must_use]
    pub fn to_model(&self) -> FirewallGroup {
        match self.group_type {
            FirewallGroupType::AddressGroup => {
                FirewallGroup::address_group(self.name.clone(), self.members.clone())
            }
            FirewallGroupType::Ipv6AddressGroup => {
                FirewallGroup::ipv6_address_group(self.name.clone(), self.members.clone())
            }
            FirewallGroupType::PortGroup => {
                FirewallGroup::port_group(self.name.clone(), self.members.clone())
            }
        }
    }
}

/// Firewall rules by ruleset.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirewallConfig {
    #[serde(default)]
    pub wan_in: Vec<FirewallRuleConfig>,
    #[serde(default)]
    pub wan_out: Vec<FirewallRuleConfig>,
    #[serde(default)]
    pub wan_local: Vec<FirewallRuleConfig>,
    #[serde(default)]
    pub lan_in: Vec<FirewallRuleConfig>,
    #[serde(default)]
    pub lan_out: Vec<FirewallRuleConfig>,
    #[serde(default)]
    pub lan_local: Vec<FirewallRuleConfig>,
    #[serde(default)]
    pub guest_in: Vec<FirewallRuleConfig>,
    #[serde(default)]
    pub guest_out: Vec<FirewallRuleConfig>,
    #[serde(default)]
    pub guest_local: Vec<FirewallRuleConfig>,
    #[serde(default)]
    pub inter_vlan: Vec<FirewallRuleConfig>,
}

impl FirewallConfig {
    /// Iterate over all rules with their ruleset.
    pub fn all_rules(&self) -> Vec<(FirewallRuleset, &FirewallRuleConfig)> {
        let mut rules = Vec::new();
        rules.extend(self.wan_in.iter().map(|r| (FirewallRuleset::WanIn, r)));
        rules.extend(self.wan_out.iter().map(|r| (FirewallRuleset::WanOut, r)));
        rules.extend(self.wan_local.iter().map(|r| (FirewallRuleset::WanLocal, r)));
        rules.extend(self.lan_in.iter().map(|r| (FirewallRuleset::LanIn, r)));
        rules.extend(self.lan_out.iter().map(|r| (FirewallRuleset::LanOut, r)));
        rules.extend(self.lan_local.iter().map(|r| (FirewallRuleset::LanLocal, r)));
        rules.extend(self.guest_in.iter().map(|r| (FirewallRuleset::GuestIn, r)));
        rules.extend(self.guest_out.iter().map(|r| (FirewallRuleset::GuestOut, r)));
        rules.extend(self.guest_local.iter().map(|r| (FirewallRuleset::GuestLocal, r)));
        rules.extend(self.inter_vlan.iter().map(|r| (FirewallRuleset::InterVlan, r)));
        rules
    }
}

/// Firewall rule configuration (name-based references).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRuleConfig {
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub rule_index: Option<i32>,
    #[serde(default)]
    pub action: FirewallAction,
    #[serde(default)]
    pub protocol: Protocol,

    #[serde(default)]
    pub logging: bool,
    #[serde(default)]
    pub state_new: bool,
    #[serde(default)]
    pub state_established: bool,
    #[serde(default)]
    pub state_invalid: bool,
    #[serde(default)]
    pub state_related: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src_network: Option<String>,
    #[serde(default)]
    pub src_firewallgroup: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src_mac_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src_port: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dst_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dst_network: Option<String>,
    #[serde(default)]
    pub dst_firewallgroup: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dst_port: Option<String>,
}

/// Port forwarding configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardConfig {
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<PortForwardSourceConfig>,
    pub dst_port: String,
    pub fwd: String,
    pub fwd_port: String,
    #[serde(default)]
    pub proto: Protocol,
    #[serde(default)]
    pub log: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pfwd_interface: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortForwardSourceConfig {
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "limited")]
    Limited {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        src_ip: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        src_firewallgroup: Option<String>,
    },
}

/// Traffic rule configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficRuleConfig {
    pub description: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub action: TrafficActionConfig,
    #[serde(default)]
    pub matching_target: MatchingTargetConfig,
    #[serde(default)]
    pub target_devices: TargetDevicesConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule: Option<ScheduleConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bandwidth_limit: Option<BandwidthLimitConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TrafficActionConfig {
    #[default]
    Block,
    Allow,
    Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MatchingTargetConfig {
    #[default]
    AllTraffic,
    App {
        app_id: String,
    },
    AppCategory {
        category_id: String,
    },
    Domain {
        domains: Vec<String>,
    },
    IpAddress {
        addresses: Vec<String>,
    },
    Region {
        regions: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TargetDevicesConfig {
    #[default]
    AllClients,
    Network {
        network: String,
    },
    Client {
        mac: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub days: Vec<crate::types::DayOfWeek>,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthLimitConfig {
    pub download_kbps: Option<u32>,
    pub upload_kbps: Option<u32>,
}

/// VPN configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wireguard: Option<WireGuardConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub site_to_site: Vec<SiteToSiteConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardConfig {
    pub name: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    pub network: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dns: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_networks: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clients: Vec<WireGuardClientConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardClientConfig {
    pub name: String,
    pub public_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preshared_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assigned_ip: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_ips: Vec<String>,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteToSiteConfig {
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub vpn_type: SiteVpnType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_ip: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remote_subnets: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_subnets: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub psk: Option<String>,
}

/// DHCP reservation config with name-based network reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhcpReservationConfig {
    pub mac: String,
    pub ip: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
}

/// DNS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    #[serde(default)]
    pub records: Vec<DnsRecordConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upstream: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filtering: Option<DnsFilteringConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecordConfig {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub record_type: crate::types::DnsRecordType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsFilteringConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub level: DnsFilterLevel,
}

/// Client policies (blocked list, groups).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientsConfig {
    #[serde(default)]
    pub blocked: Vec<BlockedClientConfig>,
    #[serde(default)]
    pub groups: Vec<ClientGroupConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedClientConfig {
    pub mac: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientGroupConfig {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qos_rate_max_down: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qos_rate_max_up: Option<i32>,
}

impl ClientGroupConfig {
    #[must_use]
    pub fn to_model(&self) -> ClientGroup {
        ClientGroup {
            id: None,
            name: self.name.clone(),
            qos_rate_max_down: self.qos_rate_max_down,
            qos_rate_max_up: self.qos_rate_max_up,
        }
    }
}

fn default_true() -> bool {
    true
}

impl PortForwardConfig {
    #[must_use]
    pub fn to_model(&self) -> PortForward {
        let mut model = PortForward::new(
            self.name.clone(),
            self.dst_port.clone(),
            self.fwd.clone(),
            self.fwd_port.clone(),
        )
        .with_protocol(self.proto);

        model.enabled = self.enabled;
        model.log = self.log;
        if let Some(interface) = &self.pfwd_interface {
            model.interface.clone_from(interface);
        }

        if let Some(src) = &self.src {
            match src {
                PortForwardSourceConfig::Any => {
                    model.src = None;
                }
                PortForwardSourceConfig::Limited { src_ip, src_firewallgroup } => {
                    if let Some(ip) = src_ip {
                        model.src = Some(ip.clone());
                    } else if let Some(group) = src_firewallgroup {
                        model.src = Some(group.clone());
                    }
                }
            }
        }

        model
    }
}

impl TrafficRuleConfig {
    #[must_use]
    pub fn to_model(&self) -> TrafficRule {
        let mut rule = match self.action {
            TrafficActionConfig::Block => TrafficRule::block(self.description.clone()),
            TrafficActionConfig::Allow => TrafficRule::allow(self.description.clone()),
            TrafficActionConfig::Limit => {
                let limit = self
                    .bandwidth_limit
                    .clone()
                    .unwrap_or(BandwidthLimitConfig { download_kbps: None, upload_kbps: None });
                TrafficRule::rate_limit(
                    self.description.clone(),
                    limit.download_kbps.unwrap_or(0),
                    limit.upload_kbps.unwrap_or(0),
                )
            }
        };

        rule.enabled = self.enabled;

        match &self.target_devices {
            TargetDevicesConfig::AllClients => {
                rule = rule.for_all_clients();
            }
            TargetDevicesConfig::Network { network } => {
                rule = rule.for_network(network.clone());
            }
            TargetDevicesConfig::Client { mac } => {
                rule = rule.for_client(mac.clone());
            }
        }

        match &self.matching_target {
            MatchingTargetConfig::AllTraffic => {
                rule.matching_target = "all_traffic".to_string();
            }
            MatchingTargetConfig::App { app_id } => {
                rule.matching_target = "app".to_string();
                rule.app_ids = vec![app_id.clone()];
            }
            MatchingTargetConfig::AppCategory { category_id } => {
                rule.matching_target = "app_category".to_string();
                rule.app_category_ids = vec![category_id.clone()];
            }
            MatchingTargetConfig::Domain { domains } => {
                rule.matching_target = "domain".to_string();
                rule.domains.clone_from(domains);
            }
            MatchingTargetConfig::IpAddress { addresses } => {
                rule.matching_target = "ip_address".to_string();
                rule.ip_addresses.clone_from(addresses);
            }
            MatchingTargetConfig::Region { regions } => {
                rule.matching_target = "region".to_string();
                rule.regions.clone_from(regions);
            }
        }

        if let Some(schedule) = &self.schedule {
            rule.schedule_days.clone_from(&schedule.days);
            rule.schedule_start = Some(schedule.start_time.clone());
            rule.schedule_end = Some(schedule.end_time.clone());
        }

        rule
    }
}

impl WireGuardConfig {
    #[must_use]
    pub fn to_server(&self) -> WireGuardServer {
        let mut server = WireGuardServer::new(
            self.name.clone().unwrap_or_else(|| "WireGuard".to_string()),
            self.network.clone(),
        );
        server.enabled = self.enabled;
        if let Some(port) = self.port {
            server.port = port;
        }
        server.dns_servers.clone_from(&self.dns);
        server.allowed_networks.clone_from(&self.allowed_networks);
        server
    }

    #[must_use]
    pub fn peers(&self) -> Vec<WireGuardPeer> {
        self.clients
            .iter()
            .map(|client| {
                let mut peer = WireGuardPeer::new(client.name.clone(), client.public_key.clone());
                peer.preshared_key.clone_from(&client.preshared_key);
                peer.allowed_ips = if client.allowed_ips.is_empty() {
                    client.assigned_ip.clone().map(|ip| vec![ip]).unwrap_or_default()
                } else {
                    client.allowed_ips.clone()
                };
                peer.enabled = client.enabled;
                peer
            })
            .collect()
    }
}

impl SiteToSiteConfig {
    #[must_use]
    pub fn to_model(&self) -> SiteVpn {
        let mut vpn = SiteVpn::new(self.name.clone(), self.remote_ip.clone().unwrap_or_default());
        vpn.enabled = self.enabled;
        vpn.vpn_type = self.vpn_type;
        vpn.remote_subnets.clone_from(&self.remote_subnets);
        vpn.local_subnets.clone_from(&self.local_subnets);
        vpn.psk.clone_from(&self.psk);
        vpn
    }
}

impl DhcpReservationConfig {
    #[must_use]
    pub fn to_model(&self, network_id: Option<String>) -> DhcpReservation {
        let mut reservation = DhcpReservation::new(self.mac.clone(), self.ip.clone());
        if let Some(name) = &self.name {
            reservation = reservation.with_name(name.clone());
        }
        if let Some(network_id) = network_id {
            reservation = reservation.with_network(network_id);
        }
        reservation
    }
}

impl DnsRecordConfig {
    #[must_use]
    pub fn to_model(&self) -> DnsRecord {
        let mut record = DnsRecord::a_record(self.key.clone(), self.value.clone());
        record.record_type = self.record_type;
        record.ttl = self.ttl;
        record
    }
}

impl FirewallRuleConfig {
    #[must_use]
    pub fn to_model(
        &self,
        ruleset: FirewallRuleset,
        network_ids: &HashMap<String, String>,
        group_ids: &HashMap<String, String>,
    ) -> FirewallRule {
        let mut rule = FirewallRule::new(self.name.clone(), ruleset, self.action);
        rule.enabled = self.enabled;
        if let Some(index) = self.rule_index {
            rule.rule_index = index;
        }
        rule.protocol = self.protocol;
        rule.logging = self.logging;
        rule.state_new = self.state_new;
        rule.state_established = self.state_established;
        rule.state_invalid = self.state_invalid;
        rule.state_related = self.state_related;
        rule.src_address.clone_from(&self.src_address);
        rule.src_mac_address.clone_from(&self.src_mac_address);
        rule.src_port.clone_from(&self.src_port);
        rule.dst_address.clone_from(&self.dst_address);
        rule.dst_port.clone_from(&self.dst_port);

        if let Some(src_network) = &self.src_network
            && let Some(id) = network_ids.get(src_network)
        {
            rule.src_networkconf_id = Some(id.clone());
            rule.src_networkconf_type = Some("NETv4".to_string());
        }

        if let Some(dst_network) = &self.dst_network
            && let Some(id) = network_ids.get(dst_network)
        {
            rule.dst_networkconf_id = Some(id.clone());
            rule.dst_networkconf_type = Some("NETv4".to_string());
        }

        rule.src_firewallgroup_ids =
            self.src_firewallgroup.iter().filter_map(|name| group_ids.get(name).cloned()).collect();
        rule.dst_firewallgroup_ids =
            self.dst_firewallgroup.iter().filter_map(|name| group_ids.get(name).cloned()).collect();

        rule
    }
}
