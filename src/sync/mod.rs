//! Declarative sync engine for UniFi configuration.

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::client::UnifiClient;
use crate::config::schema::{
    ClientsConfig, DnsConfig, FirewallRuleConfig, UnifiDeclarativeConfig, WireGuardConfig,
};
use crate::error::{Result, UnifiError};
use crate::models::{
    Client, ClientGroup, DhcpReservation, DnsRecord, FirewallGroup, FirewallRule, Network,
    PortForward, SiteVpn, TrafficRule, WireGuardPeer, WireGuardServer,
};
use crate::services::{
    ClientService, DhcpService, DnsService, FirewallService, NetworkService, PortForwardService,
    TrafficService, VpnService,
};
use crate::types::FirewallRuleset;

/// Sync action types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAction {
    Create,
    Update,
    Delete,
    Block,
    Unblock,
    Noop,
}

/// Resource kinds supported by sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Network,
    FirewallGroup,
    FirewallRule,
    PortForward,
    TrafficRule,
    WireGuardServer,
    WireGuardPeer,
    SiteVpn,
    DhcpReservation,
    DnsRecord,
    ClientGroup,
    BlockedClient,
}

/// Change record produced by diff or apply.
#[derive(Debug, Clone)]
pub struct SyncChange {
    pub resource: ResourceKind,
    pub name: String,
    pub action: SyncAction,
    pub details: Option<String>,
}

/// Sync plan (diff output).
#[derive(Debug, Clone, Default)]
pub struct SyncPlan {
    pub changes: Vec<SyncChange>,
}

/// Sync options.
#[derive(Debug, Clone, Copy, Default)]
pub struct SyncOptions {
    pub dry_run: bool,
    pub prune: bool,
}

/// Rollback action for applied changes.
#[derive(Debug, Clone)]
pub enum RollbackAction {
    Delete { resource: ResourceKind, id: String },
    Upsert { resource: ResourceKind, payload: serde_json::Value },
    SetClientBlocked { mac: String, blocked: bool },
}

/// Sync report containing changes and rollback info.
#[derive(Debug, Default)]
pub struct SyncReport {
    pub changes: Vec<SyncChange>,
    pub rollback: Vec<RollbackAction>,
}

impl SyncReport {
    /// Roll back applied changes (best-effort).
    pub async fn rollback(&self, client: &UnifiClient) -> Result<()> {
        for action in self.rollback.iter().rev() {
            apply_rollback_action(client, action).await?;
        }
        Ok(())
    }
}

/// Compute a diff plan without applying changes.
pub async fn diff(client: &UnifiClient, config: &UnifiDeclarativeConfig) -> Result<SyncPlan> {
    let mut report = SyncReport::default();
    apply_all(client, config, SyncOptions { dry_run: true, prune: false }, &mut report).await?;
    Ok(SyncPlan { changes: report.changes })
}

/// Apply configuration changes.
pub async fn apply(
    client: &UnifiClient,
    config: &UnifiDeclarativeConfig,
    options: SyncOptions,
) -> Result<SyncReport> {
    let mut report = SyncReport::default();
    apply_all(client, config, options, &mut report).await?;
    Ok(report)
}

async fn apply_all(
    client: &UnifiClient,
    config: &UnifiDeclarativeConfig,
    options: SyncOptions,
    report: &mut SyncReport,
) -> Result<()> {
    let networks = sync_networks(client, config, options, report).await?;
    let firewall_groups = sync_firewall_groups(client, config, options, report).await?;
    sync_firewall_rules(client, config, options, report, &networks, &firewall_groups).await?;
    sync_port_forwards(client, config, options, report).await?;
    sync_traffic_rules(client, config, options, report).await?;
    sync_vpn(client, config, options, report).await?;
    sync_dhcp_reservations(client, config, options, report, &networks).await?;
    sync_dns(client, config, options, report).await?;
    sync_client_groups(client, config, options, report).await?;
    sync_blocked_clients(client, config.clients.as_ref(), options, report).await?;
    Ok(())
}

fn normalize_value<T: Serialize>(value: &T) -> Result<serde_json::Value> {
    let mut json = serde_json::to_value(value).map_err(UnifiError::JsonError)?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("_id");
        obj.remove("id");
        obj.remove("site_id");
        obj.remove("site");
    }
    Ok(json)
}

fn values_equal<T: Serialize>(left: &T, right: &T) -> Result<bool> {
    Ok(normalize_value(left)? == normalize_value(right)?)
}

async fn sync_networks(
    client: &UnifiClient,
    config: &UnifiDeclarativeConfig,
    options: SyncOptions,
    report: &mut SyncReport,
) -> Result<HashMap<String, String>> {
    let service = NetworkService::new(client);
    let existing = service.list().await?;
    let by_name: HashMap<String, Network> =
        existing.into_iter().map(|n| (n.name.clone(), n)).collect();

    let mut seen = HashSet::new();
    let mut id_map = HashMap::new();

    for desired in &config.networks {
        let desired_model = desired.to_model();
        if let Some(current) = by_name.get(&desired.name) {
            seen.insert(current.name.clone());
            if let Some(id) = current.id.clone() {
                id_map.insert(current.name.clone(), id);
            }
            if !values_equal(&desired_model, current)? {
                report.changes.push(SyncChange {
                    resource: ResourceKind::Network,
                    name: desired.name.clone(),
                    action: SyncAction::Update,
                    details: None,
                });
                if !options.dry_run
                    && let Some(id) = current.id.clone()
                {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::Network,
                        payload: serde_json::to_value(current).map_err(UnifiError::JsonError)?,
                    });
                    let mut update = desired_model;
                    update.id = Some(id.clone());
                    service.update(&id, &update).await?;
                }
            }
        } else {
            report.changes.push(SyncChange {
                resource: ResourceKind::Network,
                name: desired.name.clone(),
                action: SyncAction::Create,
                details: None,
            });
            if !options.dry_run {
                let created = service.create(&desired_model).await?;
                if let Some(id) = created.id.clone() {
                    id_map.insert(desired.name.clone(), id.clone());
                    report
                        .rollback
                        .push(RollbackAction::Delete { resource: ResourceKind::Network, id });
                }
            }
        }
    }

    if options.prune {
        for (name, current) in by_name {
            if !seen.contains(&name)
                && let Some(id) = current.id.clone()
            {
                report.changes.push(SyncChange {
                    resource: ResourceKind::Network,
                    name: name.clone(),
                    action: SyncAction::Delete,
                    details: None,
                });
                if !options.dry_run {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::Network,
                        payload: serde_json::to_value(&current).map_err(UnifiError::JsonError)?,
                    });
                    service.delete(&id).await?;
                }
            }
        }
    }

    if id_map.is_empty() {
        let refreshed = service.list().await?;
        for network in refreshed {
            if let Some(id) = network.id.clone() {
                id_map.insert(network.name.clone(), id);
            }
        }
    }

    Ok(id_map)
}

async fn sync_firewall_groups(
    client: &UnifiClient,
    config: &UnifiDeclarativeConfig,
    options: SyncOptions,
    report: &mut SyncReport,
) -> Result<HashMap<String, String>> {
    let service = FirewallService::new(client);
    let existing = service.list_groups().await?;
    let by_name: HashMap<String, FirewallGroup> =
        existing.into_iter().map(|g| (g.name.clone(), g)).collect();
    let mut seen = HashSet::new();
    let mut id_map = HashMap::new();

    for desired in &config.firewall_groups {
        let desired_model = desired.to_model();
        if let Some(current) = by_name.get(&desired.name) {
            seen.insert(current.name.clone());
            if let Some(id) = current.id.clone() {
                id_map.insert(current.name.clone(), id);
            }
            if !values_equal(&desired_model, current)? {
                report.changes.push(SyncChange {
                    resource: ResourceKind::FirewallGroup,
                    name: desired.name.clone(),
                    action: SyncAction::Update,
                    details: None,
                });
                if !options.dry_run
                    && let Some(id) = current.id.clone()
                {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::FirewallGroup,
                        payload: serde_json::to_value(current).map_err(UnifiError::JsonError)?,
                    });
                    let mut update = desired_model;
                    update.id = Some(id.clone());
                    service.update_group(&id, &update).await?;
                }
            }
        } else {
            report.changes.push(SyncChange {
                resource: ResourceKind::FirewallGroup,
                name: desired.name.clone(),
                action: SyncAction::Create,
                details: None,
            });
            if !options.dry_run {
                let created = service.create_group(&desired_model).await?;
                if let Some(id) = created.id.clone() {
                    id_map.insert(desired.name.clone(), id.clone());
                    report
                        .rollback
                        .push(RollbackAction::Delete { resource: ResourceKind::FirewallGroup, id });
                }
            }
        }
    }

    if options.prune {
        for (name, current) in by_name {
            if !seen.contains(&name)
                && let Some(id) = current.id.clone()
            {
                report.changes.push(SyncChange {
                    resource: ResourceKind::FirewallGroup,
                    name: name.clone(),
                    action: SyncAction::Delete,
                    details: None,
                });
                if !options.dry_run {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::FirewallGroup,
                        payload: serde_json::to_value(&current).map_err(UnifiError::JsonError)?,
                    });
                    service.delete_group(&id).await?;
                }
            }
        }
    }

    if id_map.is_empty() {
        let refreshed = service.list_groups().await?;
        for group in refreshed {
            if let Some(id) = group.id.clone() {
                id_map.insert(group.name.clone(), id);
            }
        }
    }

    Ok(id_map)
}

async fn sync_firewall_rules(
    client: &UnifiClient,
    config: &UnifiDeclarativeConfig,
    options: SyncOptions,
    report: &mut SyncReport,
    network_ids: &HashMap<String, String>,
    group_ids: &HashMap<String, String>,
) -> Result<()> {
    let service = FirewallService::new(client);
    let existing = service.list_rules().await?;
    let mut by_key: HashMap<String, FirewallRule> = HashMap::new();
    for rule in existing {
        let key = format!("{}::{}", rule.ruleset, rule.name);
        by_key.insert(key, rule);
    }

    let mut seen = HashSet::new();

    for (ruleset, rule_cfg) in config.firewall.all_rules() {
        let desired_model = rule_cfg.to_model(ruleset, network_ids, group_ids);
        let key = format!("{}::{}", ruleset, rule_cfg.name);
        if let Some(current) = by_key.get(&key) {
            seen.insert(key.clone());
            if !values_equal(&desired_model, current)? {
                report.changes.push(SyncChange {
                    resource: ResourceKind::FirewallRule,
                    name: rule_cfg.name.clone(),
                    action: SyncAction::Update,
                    details: Some(ruleset.to_string()),
                });
                if !options.dry_run
                    && let Some(id) = current.id.clone()
                {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::FirewallRule,
                        payload: serde_json::to_value(current).map_err(UnifiError::JsonError)?,
                    });
                    let mut update = desired_model;
                    update.id = Some(id.clone());
                    service.update_rule(&id, &update).await?;
                }
            }
        } else {
            report.changes.push(SyncChange {
                resource: ResourceKind::FirewallRule,
                name: rule_cfg.name.clone(),
                action: SyncAction::Create,
                details: Some(ruleset.to_string()),
            });
            if !options.dry_run {
                let created = service.create_rule(&desired_model).await?;
                if let Some(id) = created.id.clone() {
                    report
                        .rollback
                        .push(RollbackAction::Delete { resource: ResourceKind::FirewallRule, id });
                }
            }
        }
    }

    if options.prune {
        for (key, current) in by_key {
            if !seen.contains(&key)
                && let Some(id) = current.id.clone()
            {
                report.changes.push(SyncChange {
                    resource: ResourceKind::FirewallRule,
                    name: current.name.clone(),
                    action: SyncAction::Delete,
                    details: Some(current.ruleset.to_string()),
                });
                if !options.dry_run {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::FirewallRule,
                        payload: serde_json::to_value(&current).map_err(UnifiError::JsonError)?,
                    });
                    service.delete_rule(&id).await?;
                }
            }
        }
    }

    Ok(())
}

async fn sync_port_forwards(
    client: &UnifiClient,
    config: &UnifiDeclarativeConfig,
    options: SyncOptions,
    report: &mut SyncReport,
) -> Result<()> {
    let service = PortForwardService::new(client);
    let existing = service.list().await?;
    let by_name: HashMap<String, PortForward> =
        existing.into_iter().map(|pf| (pf.name.clone(), pf)).collect();
    let mut seen = HashSet::new();

    for desired in &config.port_forward {
        let desired_model = desired.to_model();
        if let Some(current) = by_name.get(&desired.name) {
            seen.insert(current.name.clone());
            if !values_equal(&desired_model, current)? {
                report.changes.push(SyncChange {
                    resource: ResourceKind::PortForward,
                    name: desired.name.clone(),
                    action: SyncAction::Update,
                    details: None,
                });
                if !options.dry_run
                    && let Some(id) = current.id.clone()
                {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::PortForward,
                        payload: serde_json::to_value(current).map_err(UnifiError::JsonError)?,
                    });
                    let mut update = desired_model;
                    update.id = Some(id.clone());
                    service.update(&id, &update).await?;
                }
            }
        } else {
            report.changes.push(SyncChange {
                resource: ResourceKind::PortForward,
                name: desired.name.clone(),
                action: SyncAction::Create,
                details: None,
            });
            if !options.dry_run {
                let created = service.create(&desired_model).await?;
                if let Some(id) = created.id.clone() {
                    report
                        .rollback
                        .push(RollbackAction::Delete { resource: ResourceKind::PortForward, id });
                }
            }
        }
    }

    if options.prune {
        for (name, current) in by_name {
            if !seen.contains(&name)
                && let Some(id) = current.id.clone()
            {
                report.changes.push(SyncChange {
                    resource: ResourceKind::PortForward,
                    name: name.clone(),
                    action: SyncAction::Delete,
                    details: None,
                });
                if !options.dry_run {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::PortForward,
                        payload: serde_json::to_value(&current).map_err(UnifiError::JsonError)?,
                    });
                    service.delete(&id).await?;
                }
            }
        }
    }

    Ok(())
}

async fn sync_traffic_rules(
    client: &UnifiClient,
    config: &UnifiDeclarativeConfig,
    options: SyncOptions,
    report: &mut SyncReport,
) -> Result<()> {
    let service = TrafficService::new(client);
    let existing = service.list().await?;
    let by_desc: HashMap<String, TrafficRule> =
        existing.into_iter().map(|rule| (rule.description.clone(), rule)).collect();

    let seen = upsert_traffic_rules(&service, config, options, report, &by_desc).await?;
    prune_traffic_rules(&service, options, report, &by_desc, &seen).await?;
    Ok(())
}

async fn upsert_traffic_rules(
    service: &TrafficService<'_>,
    config: &UnifiDeclarativeConfig,
    options: SyncOptions,
    report: &mut SyncReport,
    by_desc: &HashMap<String, TrafficRule>,
) -> Result<HashSet<String>> {
    let mut seen = HashSet::new();
    for desired in &config.traffic_rules {
        let desired_model = desired.to_model();
        if let Some(current) = by_desc.get(&desired.description) {
            seen.insert(current.description.clone());
            if !values_equal(&desired_model, current)? {
                report.changes.push(SyncChange {
                    resource: ResourceKind::TrafficRule,
                    name: desired.description.clone(),
                    action: SyncAction::Update,
                    details: None,
                });
                if !options.dry_run
                    && let Some(id) = current.id.clone()
                {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::TrafficRule,
                        payload: serde_json::to_value(current).map_err(UnifiError::JsonError)?,
                    });
                    let mut update = desired_model;
                    update.id = Some(id.clone());
                    service.update(&id, &update).await?;
                }
            }
        } else {
            report.changes.push(SyncChange {
                resource: ResourceKind::TrafficRule,
                name: desired.description.clone(),
                action: SyncAction::Create,
                details: None,
            });
            if !options.dry_run {
                let created = service.create(&desired_model).await?;
                if let Some(id) = created.id.clone() {
                    report
                        .rollback
                        .push(RollbackAction::Delete { resource: ResourceKind::TrafficRule, id });
                }
            }
        }
    }
    Ok(seen)
}

async fn prune_traffic_rules(
    service: &TrafficService<'_>,
    options: SyncOptions,
    report: &mut SyncReport,
    by_desc: &HashMap<String, TrafficRule>,
    seen: &HashSet<String>,
) -> Result<()> {
    if !options.prune {
        return Ok(());
    }
    for (desc, current) in by_desc {
        if !seen.contains(desc)
            && let Some(id) = current.id.clone()
        {
            report.changes.push(SyncChange {
                resource: ResourceKind::TrafficRule,
                name: desc.clone(),
                action: SyncAction::Delete,
                details: None,
            });
            if !options.dry_run {
                report.rollback.push(RollbackAction::Upsert {
                    resource: ResourceKind::TrafficRule,
                    payload: serde_json::to_value(current).map_err(UnifiError::JsonError)?,
                });
                service.delete(&id).await?;
            }
        }
    }
    Ok(())
}

async fn sync_vpn(
    client: &UnifiClient,
    config: &UnifiDeclarativeConfig,
    options: SyncOptions,
    report: &mut SyncReport,
) -> Result<()> {
    let Some(vpn_config) = &config.vpn else {
        return Ok(());
    };
    let service = VpnService::new(client);

    if let Some(wireguard) = &vpn_config.wireguard {
        sync_wireguard(client, wireguard, options, report).await?;
    }

    let existing = service.list_site_vpns().await?;
    let by_name: HashMap<String, SiteVpn> =
        existing.into_iter().map(|vpn| (vpn.name.clone(), vpn)).collect();
    let mut seen = HashSet::new();

    for desired in &vpn_config.site_to_site {
        let desired_model = desired.to_model();
        if let Some(current) = by_name.get(&desired.name) {
            seen.insert(current.name.clone());
            if !values_equal(&desired_model, current)? {
                report.changes.push(SyncChange {
                    resource: ResourceKind::SiteVpn,
                    name: desired.name.clone(),
                    action: SyncAction::Update,
                    details: None,
                });
                if !options.dry_run
                    && let Some(id) = current.id.clone()
                {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::SiteVpn,
                        payload: serde_json::to_value(current).map_err(UnifiError::JsonError)?,
                    });
                    let mut update = desired_model;
                    update.id = Some(id.clone());
                    service.update_site_vpn(&id, &update).await?;
                }
            }
        } else {
            report.changes.push(SyncChange {
                resource: ResourceKind::SiteVpn,
                name: desired.name.clone(),
                action: SyncAction::Create,
                details: None,
            });
            if !options.dry_run {
                let created = service.create_site_vpn(&desired_model).await?;
                if let Some(id) = created.id.clone() {
                    report
                        .rollback
                        .push(RollbackAction::Delete { resource: ResourceKind::SiteVpn, id });
                }
            }
        }
    }

    if options.prune {
        for (name, current) in by_name {
            if !seen.contains(&name)
                && let Some(id) = current.id.clone()
            {
                report.changes.push(SyncChange {
                    resource: ResourceKind::SiteVpn,
                    name: name.clone(),
                    action: SyncAction::Delete,
                    details: None,
                });
                if !options.dry_run {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::SiteVpn,
                        payload: serde_json::to_value(&current).map_err(UnifiError::JsonError)?,
                    });
                    service.delete_site_vpn(&id).await?;
                }
            }
        }
    }

    Ok(())
}

async fn sync_wireguard(
    client: &UnifiClient,
    wireguard: &WireGuardConfig,
    options: SyncOptions,
    report: &mut SyncReport,
) -> Result<()> {
    let service = VpnService::new(client);
    let server_id = sync_wireguard_server(&service, wireguard, options, report).await?;
    sync_wireguard_peers(&service, wireguard, server_id, options, report).await?;
    Ok(())
}

async fn sync_wireguard_server(
    service: &VpnService<'_>,
    wireguard: &WireGuardConfig,
    options: SyncOptions,
    report: &mut SyncReport,
) -> Result<Option<String>> {
    let existing_servers = service.list_wireguard_servers().await?;
    let by_name: HashMap<String, WireGuardServer> =
        existing_servers.into_iter().map(|server| (server.name.clone(), server)).collect();

    let desired_server = wireguard.to_server();
    let server_name = desired_server.name.clone();
    let current = by_name.get(&server_name);

    if let Some(current) = current {
        if !values_equal(&desired_server, current)? {
            report.changes.push(SyncChange {
                resource: ResourceKind::WireGuardServer,
                name: server_name.clone(),
                action: SyncAction::Update,
                details: None,
            });
            if !options.dry_run
                && let Some(id) = current.id.clone()
            {
                report.rollback.push(RollbackAction::Upsert {
                    resource: ResourceKind::WireGuardServer,
                    payload: serde_json::to_value(current).map_err(UnifiError::JsonError)?,
                });
                let mut update = desired_server.clone();
                update.id = Some(id.clone());
                service.update_wireguard_server(&id, &update).await?;
            }
        }
        return Ok(current.id.clone());
    }

    report.changes.push(SyncChange {
        resource: ResourceKind::WireGuardServer,
        name: server_name.clone(),
        action: SyncAction::Create,
        details: None,
    });
    if options.dry_run {
        return Ok(None);
    }

    let created = service.create_wireguard_server(&desired_server).await?;
    if let Some(id) = created.id.as_ref() {
        report.rollback.push(RollbackAction::Delete {
            resource: ResourceKind::WireGuardServer,
            id: id.clone(),
        });
    }
    Ok(created.id)
}

async fn sync_wireguard_peers(
    service: &VpnService<'_>,
    wireguard: &WireGuardConfig,
    server_id: Option<String>,
    options: SyncOptions,
    report: &mut SyncReport,
) -> Result<()> {
    let existing_peers = service.list_wireguard_peers().await?;
    let peers_by_name: HashMap<String, WireGuardPeer> =
        existing_peers.into_iter().map(|peer| (peer.name.clone(), peer)).collect();
    let mut seen = HashSet::new();

    for mut desired_peer in wireguard.peers() {
        if let Some(server_id) = server_id.as_ref() {
            desired_peer.server_id = Some(server_id.clone());
        }
        if let Some(current) = peers_by_name.get(&desired_peer.name) {
            seen.insert(current.name.clone());
            if !values_equal(&desired_peer, current)? {
                report.changes.push(SyncChange {
                    resource: ResourceKind::WireGuardPeer,
                    name: desired_peer.name.clone(),
                    action: SyncAction::Update,
                    details: None,
                });
                if !options.dry_run
                    && let Some(id) = current.id.clone()
                {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::WireGuardPeer,
                        payload: serde_json::to_value(current).map_err(UnifiError::JsonError)?,
                    });
                    let mut update = desired_peer.clone();
                    update.id = Some(id.clone());
                    service.update_wireguard_peer(&id, &update).await?;
                }
            }
        } else {
            report.changes.push(SyncChange {
                resource: ResourceKind::WireGuardPeer,
                name: desired_peer.name.clone(),
                action: SyncAction::Create,
                details: None,
            });
            if !options.dry_run {
                let created = service.create_wireguard_peer(&desired_peer).await?;
                if let Some(id) = created.id {
                    report
                        .rollback
                        .push(RollbackAction::Delete { resource: ResourceKind::WireGuardPeer, id });
                }
            }
        }
    }

    if options.prune {
        for (name, current) in peers_by_name {
            if !seen.contains(&name)
                && let Some(id) = current.id.clone()
            {
                report.changes.push(SyncChange {
                    resource: ResourceKind::WireGuardPeer,
                    name: name.clone(),
                    action: SyncAction::Delete,
                    details: None,
                });
                if !options.dry_run {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::WireGuardPeer,
                        payload: serde_json::to_value(&current).map_err(UnifiError::JsonError)?,
                    });
                    service.delete_wireguard_peer(&id).await?;
                }
            }
        }
    }

    Ok(())
}

async fn sync_dhcp_reservations(
    client: &UnifiClient,
    config: &UnifiDeclarativeConfig,
    options: SyncOptions,
    report: &mut SyncReport,
    network_ids: &HashMap<String, String>,
) -> Result<()> {
    let service = DhcpService::new(client);
    let existing = service.list_reservations().await?;
    let by_mac: HashMap<String, DhcpReservation> =
        existing.into_iter().map(|res| (res.mac_address.clone(), res)).collect();
    let mut seen = HashSet::new();

    for desired in &config.dhcp_reservations {
        let network_id = desired.network.as_ref().and_then(|name| network_ids.get(name).cloned());
        let desired_model = desired.to_model(network_id);
        if let Some(current) = by_mac.get(&desired.mac) {
            seen.insert(current.mac_address.clone());
            if !values_equal(&desired_model, current)? {
                report.changes.push(SyncChange {
                    resource: ResourceKind::DhcpReservation,
                    name: desired.mac.clone(),
                    action: SyncAction::Update,
                    details: None,
                });
                if !options.dry_run {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::DhcpReservation,
                        payload: serde_json::to_value(current).map_err(UnifiError::JsonError)?,
                    });
                    service.update_reservation(&desired.mac, &desired.ip).await?;
                }
            }
        } else {
            report.changes.push(SyncChange {
                resource: ResourceKind::DhcpReservation,
                name: desired.mac.clone(),
                action: SyncAction::Create,
                details: None,
            });
            if !options.dry_run {
                let created = service.create_reservation(&desired_model).await?;
                report.rollback.push(RollbackAction::Delete {
                    resource: ResourceKind::DhcpReservation,
                    id: created.mac_address.clone(),
                });
            }
        }
    }

    if options.prune {
        for (mac, current) in by_mac {
            if !seen.contains(&mac) {
                report.changes.push(SyncChange {
                    resource: ResourceKind::DhcpReservation,
                    name: mac.clone(),
                    action: SyncAction::Delete,
                    details: None,
                });
                if !options.dry_run {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::DhcpReservation,
                        payload: serde_json::to_value(&current).map_err(UnifiError::JsonError)?,
                    });
                    service.delete_reservation(&mac).await?;
                }
            }
        }
    }

    Ok(())
}

async fn sync_dns(
    client: &UnifiClient,
    config: &UnifiDeclarativeConfig,
    options: SyncOptions,
    report: &mut SyncReport,
) -> Result<()> {
    let Some(dns) = &config.dns else {
        return Ok(());
    };

    let service = DnsService::new(client);
    let existing = service.list().await?;
    let by_key: HashMap<String, DnsRecord> =
        existing.into_iter().map(|record| (record.key.clone(), record)).collect();
    let mut seen = HashSet::new();

    for desired in &dns.records {
        let desired_model = desired.to_model();
        if let Some(current) = by_key.get(&desired.key) {
            seen.insert(current.key.clone());
            if !values_equal(&desired_model, current)? {
                report.changes.push(SyncChange {
                    resource: ResourceKind::DnsRecord,
                    name: desired.key.clone(),
                    action: SyncAction::Update,
                    details: None,
                });
                if !options.dry_run
                    && let Some(id) = current.id.clone()
                {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::DnsRecord,
                        payload: serde_json::to_value(current).map_err(UnifiError::JsonError)?,
                    });
                    let mut update = desired_model;
                    update.id = Some(id.clone());
                    service.update(&id, &update).await?;
                }
            }
        } else {
            report.changes.push(SyncChange {
                resource: ResourceKind::DnsRecord,
                name: desired.key.clone(),
                action: SyncAction::Create,
                details: None,
            });
            if !options.dry_run {
                let created = service.create(&desired_model).await?;
                if let Some(id) = created.id.clone() {
                    report
                        .rollback
                        .push(RollbackAction::Delete { resource: ResourceKind::DnsRecord, id });
                }
            }
        }
    }

    if options.prune {
        for (key, current) in by_key {
            if !seen.contains(&key)
                && let Some(id) = current.id.clone()
            {
                report.changes.push(SyncChange {
                    resource: ResourceKind::DnsRecord,
                    name: key.clone(),
                    action: SyncAction::Delete,
                    details: None,
                });
                if !options.dry_run {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::DnsRecord,
                        payload: serde_json::to_value(&current).map_err(UnifiError::JsonError)?,
                    });
                    service.delete(&id).await?;
                }
            }
        }
    }

    if !dns.upstream.is_empty() && !options.dry_run {
        service.set_upstream_dns(&dns.upstream).await?;
    }

    if let Some(filtering) = &dns.filtering
        && !options.dry_run
    {
        service.set_dns_filtering(filtering.enabled, filtering.level).await?;
    }

    Ok(())
}

async fn sync_client_groups(
    client: &UnifiClient,
    config: &UnifiDeclarativeConfig,
    options: SyncOptions,
    report: &mut SyncReport,
) -> Result<()> {
    let Some(clients) = &config.clients else {
        return Ok(());
    };

    let service = ClientService::new(client);
    let existing = service.list_groups().await?;
    let by_name: HashMap<String, ClientGroup> =
        existing.into_iter().map(|group| (group.name.clone(), group)).collect();
    let mut seen = HashSet::new();

    for desired in &clients.groups {
        let desired_model = desired.to_model();
        if let Some(current) = by_name.get(&desired.name) {
            seen.insert(current.name.clone());
            if !values_equal(&desired_model, current)? {
                report.changes.push(SyncChange {
                    resource: ResourceKind::ClientGroup,
                    name: desired.name.clone(),
                    action: SyncAction::Update,
                    details: None,
                });
                if !options.dry_run
                    && let Some(id) = current.id.clone()
                {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::ClientGroup,
                        payload: serde_json::to_value(current).map_err(UnifiError::JsonError)?,
                    });
                    let mut update = desired_model;
                    update.id = Some(id.clone());
                    service.update_group(&id, &update).await?;
                }
            }
        } else {
            report.changes.push(SyncChange {
                resource: ResourceKind::ClientGroup,
                name: desired.name.clone(),
                action: SyncAction::Create,
                details: None,
            });
            if !options.dry_run {
                let created = service.create_group(&desired_model).await?;
                if let Some(id) = created.id.clone() {
                    report
                        .rollback
                        .push(RollbackAction::Delete { resource: ResourceKind::ClientGroup, id });
                }
            }
        }
    }

    if options.prune {
        for (name, current) in by_name {
            if !seen.contains(&name)
                && let Some(id) = current.id.clone()
            {
                report.changes.push(SyncChange {
                    resource: ResourceKind::ClientGroup,
                    name: name.clone(),
                    action: SyncAction::Delete,
                    details: None,
                });
                if !options.dry_run {
                    report.rollback.push(RollbackAction::Upsert {
                        resource: ResourceKind::ClientGroup,
                        payload: serde_json::to_value(&current).map_err(UnifiError::JsonError)?,
                    });
                    service.delete_group(&id).await?;
                }
            }
        }
    }

    Ok(())
}

async fn sync_blocked_clients(
    client: &UnifiClient,
    clients: Option<&ClientsConfig>,
    options: SyncOptions,
    report: &mut SyncReport,
) -> Result<()> {
    let Some(clients) = clients else {
        return Ok(());
    };

    let service = ClientService::new(client);
    let desired: HashSet<String> =
        clients.blocked.iter().map(|entry| entry.mac.to_lowercase()).collect();
    let existing: HashSet<String> = service
        .list_blocked()
        .await?
        .into_iter()
        .map(|client| client.mac_address.to_lowercase())
        .collect();

    for mac in desired.difference(&existing) {
        report.changes.push(SyncChange {
            resource: ResourceKind::BlockedClient,
            name: mac.clone(),
            action: SyncAction::Block,
            details: None,
        });
        if !options.dry_run {
            report
                .rollback
                .push(RollbackAction::SetClientBlocked { mac: mac.clone(), blocked: false });
            service.block(mac).await?;
        }
    }

    if options.prune {
        for mac in existing.difference(&desired) {
            report.changes.push(SyncChange {
                resource: ResourceKind::BlockedClient,
                name: mac.clone(),
                action: SyncAction::Unblock,
                details: None,
            });
            if !options.dry_run {
                report
                    .rollback
                    .push(RollbackAction::SetClientBlocked { mac: mac.clone(), blocked: true });
                service.unblock(mac).await?;
            }
        }
    }

    Ok(())
}

async fn apply_rollback_action(client: &UnifiClient, action: &RollbackAction) -> Result<()> {
    match action {
        RollbackAction::Delete { resource, id } => match resource {
            ResourceKind::Network => NetworkService::new(client).delete(id).await,
            ResourceKind::FirewallGroup => FirewallService::new(client).delete_group(id).await,
            ResourceKind::FirewallRule => FirewallService::new(client).delete_rule(id).await,
            ResourceKind::PortForward => PortForwardService::new(client).delete(id).await,
            ResourceKind::TrafficRule => TrafficService::new(client).delete(id).await,
            ResourceKind::WireGuardServer => {
                VpnService::new(client).delete_wireguard_server(id).await
            }
            ResourceKind::WireGuardPeer => VpnService::new(client).delete_wireguard_peer(id).await,
            ResourceKind::SiteVpn => VpnService::new(client).delete_site_vpn(id).await,
            ResourceKind::DhcpReservation => DhcpService::new(client).delete_reservation(id).await,
            ResourceKind::DnsRecord => DnsService::new(client).delete(id).await,
            ResourceKind::ClientGroup => ClientService::new(client).delete_group(id).await,
            ResourceKind::BlockedClient => Ok(()),
        },
        RollbackAction::Upsert { resource, payload } => match resource {
            ResourceKind::Network => upsert_network(client, payload).await,
            ResourceKind::FirewallGroup => upsert_firewall_group(client, payload).await,
            ResourceKind::FirewallRule => upsert_firewall_rule(client, payload).await,
            ResourceKind::PortForward => upsert_port_forward(client, payload).await,
            ResourceKind::TrafficRule => upsert_traffic_rule(client, payload).await,
            ResourceKind::WireGuardServer => upsert_wireguard_server(client, payload).await,
            ResourceKind::WireGuardPeer => upsert_wireguard_peer(client, payload).await,
            ResourceKind::SiteVpn => upsert_site_vpn(client, payload).await,
            ResourceKind::DhcpReservation => upsert_dhcp_reservation(client, payload).await,
            ResourceKind::DnsRecord => upsert_dns_record(client, payload).await,
            ResourceKind::ClientGroup => upsert_client_group(client, payload).await,
            ResourceKind::BlockedClient => Ok(()),
        },
        RollbackAction::SetClientBlocked { mac, blocked } => {
            let service = ClientService::new(client);
            if *blocked { service.block(mac).await } else { service.unblock(mac).await }
        }
    }
}

fn extract_id(payload: &serde_json::Value) -> Option<String> {
    payload.get("_id").and_then(|value| value.as_str()).map(str::to_string)
}

fn without_id(mut payload: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = payload.as_object_mut() {
        obj.remove("_id");
    }
    payload
}

async fn upsert_network(client: &UnifiClient, payload: &serde_json::Value) -> Result<()> {
    let service = NetworkService::new(client);
    let model: Network = serde_json::from_value(payload.clone()).map_err(UnifiError::JsonError)?;
    if let Some(id) = extract_id(payload)
        && service.update(&id, &model).await.is_ok()
    {
        return Ok(());
    }
    let model =
        serde_json::from_value(without_id(payload.clone())).map_err(UnifiError::JsonError)?;
    let _: Network = service.create(&model).await?;
    Ok(())
}

async fn upsert_firewall_group(client: &UnifiClient, payload: &serde_json::Value) -> Result<()> {
    let service = FirewallService::new(client);
    let model: FirewallGroup =
        serde_json::from_value(payload.clone()).map_err(UnifiError::JsonError)?;
    if let Some(id) = extract_id(payload)
        && service.update_group(&id, &model).await.is_ok()
    {
        return Ok(());
    }
    let model: FirewallGroup =
        serde_json::from_value(without_id(payload.clone())).map_err(UnifiError::JsonError)?;
    let _: FirewallGroup = service.create_group(&model).await?;
    Ok(())
}

async fn upsert_firewall_rule(client: &UnifiClient, payload: &serde_json::Value) -> Result<()> {
    let service = FirewallService::new(client);
    let model: FirewallRule =
        serde_json::from_value(payload.clone()).map_err(UnifiError::JsonError)?;
    if let Some(id) = extract_id(payload)
        && service.update_rule(&id, &model).await.is_ok()
    {
        return Ok(());
    }
    let model: FirewallRule =
        serde_json::from_value(without_id(payload.clone())).map_err(UnifiError::JsonError)?;
    let _: FirewallRule = service.create_rule(&model).await?;
    Ok(())
}

async fn upsert_port_forward(client: &UnifiClient, payload: &serde_json::Value) -> Result<()> {
    let service = PortForwardService::new(client);
    let model: PortForward =
        serde_json::from_value(payload.clone()).map_err(UnifiError::JsonError)?;
    if let Some(id) = extract_id(payload)
        && service.update(&id, &model).await.is_ok()
    {
        return Ok(());
    }
    let model: PortForward =
        serde_json::from_value(without_id(payload.clone())).map_err(UnifiError::JsonError)?;
    let _: PortForward = service.create(&model).await?;
    Ok(())
}

async fn upsert_traffic_rule(client: &UnifiClient, payload: &serde_json::Value) -> Result<()> {
    let service = TrafficService::new(client);
    let model: TrafficRule =
        serde_json::from_value(payload.clone()).map_err(UnifiError::JsonError)?;
    if let Some(id) = extract_id(payload)
        && service.update(&id, &model).await.is_ok()
    {
        return Ok(());
    }
    let model: TrafficRule =
        serde_json::from_value(without_id(payload.clone())).map_err(UnifiError::JsonError)?;
    let _: TrafficRule = service.create(&model).await?;
    Ok(())
}

async fn upsert_wireguard_server(client: &UnifiClient, payload: &serde_json::Value) -> Result<()> {
    let service = VpnService::new(client);
    let model: WireGuardServer =
        serde_json::from_value(payload.clone()).map_err(UnifiError::JsonError)?;
    if let Some(id) = extract_id(payload)
        && service.update_wireguard_server(&id, &model).await.is_ok()
    {
        return Ok(());
    }
    let model: WireGuardServer =
        serde_json::from_value(without_id(payload.clone())).map_err(UnifiError::JsonError)?;
    let _: WireGuardServer = service.create_wireguard_server(&model).await?;
    Ok(())
}

async fn upsert_wireguard_peer(client: &UnifiClient, payload: &serde_json::Value) -> Result<()> {
    let service = VpnService::new(client);
    let model: WireGuardPeer =
        serde_json::from_value(payload.clone()).map_err(UnifiError::JsonError)?;
    if let Some(id) = extract_id(payload)
        && service.update_wireguard_peer(&id, &model).await.is_ok()
    {
        return Ok(());
    }
    let model: WireGuardPeer =
        serde_json::from_value(without_id(payload.clone())).map_err(UnifiError::JsonError)?;
    let _: WireGuardPeer = service.create_wireguard_peer(&model).await?;
    Ok(())
}

async fn upsert_site_vpn(client: &UnifiClient, payload: &serde_json::Value) -> Result<()> {
    let service = VpnService::new(client);
    let model: SiteVpn = serde_json::from_value(payload.clone()).map_err(UnifiError::JsonError)?;
    if let Some(id) = extract_id(payload)
        && service.update_site_vpn(&id, &model).await.is_ok()
    {
        return Ok(());
    }
    let model: SiteVpn =
        serde_json::from_value(without_id(payload.clone())).map_err(UnifiError::JsonError)?;
    let _: SiteVpn = service.create_site_vpn(&model).await?;
    Ok(())
}

async fn upsert_dhcp_reservation(client: &UnifiClient, payload: &serde_json::Value) -> Result<()> {
    let service = DhcpService::new(client);
    let model: DhcpReservation =
        serde_json::from_value(payload.clone()).map_err(UnifiError::JsonError)?;
    if service.update_reservation(&model.mac_address, &model.ip_address).await.is_ok() {
        return Ok(());
    }
    let _: DhcpReservation = service.create_reservation(&model).await?;
    Ok(())
}

async fn upsert_dns_record(client: &UnifiClient, payload: &serde_json::Value) -> Result<()> {
    let service = DnsService::new(client);
    let model: DnsRecord =
        serde_json::from_value(payload.clone()).map_err(UnifiError::JsonError)?;
    if let Some(id) = extract_id(payload)
        && service.update(&id, &model).await.is_ok()
    {
        return Ok(());
    }
    let model: DnsRecord =
        serde_json::from_value(without_id(payload.clone())).map_err(UnifiError::JsonError)?;
    let _: DnsRecord = service.create(&model).await?;
    Ok(())
}

async fn upsert_client_group(client: &UnifiClient, payload: &serde_json::Value) -> Result<()> {
    let service = ClientService::new(client);
    let model: ClientGroup =
        serde_json::from_value(payload.clone()).map_err(UnifiError::JsonError)?;
    if let Some(id) = extract_id(payload)
        && service.update_group(&id, &model).await.is_ok()
    {
        return Ok(());
    }
    let model: ClientGroup =
        serde_json::from_value(without_id(payload.clone())).map_err(UnifiError::JsonError)?;
    let _: ClientGroup = service.create_group(&model).await?;
    Ok(())
}

fn export_unifi_config(client: &UnifiClient) -> crate::config::schema::UnifiConnectionConfig {
    let config = client.config();
    crate::config::schema::UnifiConnectionConfig {
        host: config.host.clone(),
        username: "${UNIFI_USERNAME}".to_string(),
        password: "${UNIFI_PASSWORD}".to_string(),
        site: config.site.clone(),
        verify_ssl: config.verify_ssl,
        timeout_secs: config.timeout_secs,
    }
}

fn export_networks(networks: Vec<Network>) -> Vec<crate::config::schema::NetworkConfig> {
    networks
        .into_iter()
        .map(|net| crate::config::schema::NetworkConfig {
            name: net.name,
            purpose: net.purpose,
            vlan_enabled: net.vlan_enabled,
            vlan: net.vlan,
            subnet: net.subnet,
            gateway: net.gateway,
            dhcp: Some(crate::config::schema::DhcpConfig {
                enabled: net.dhcp_enabled,
                start: net.dhcp_start,
                stop: net.dhcp_stop,
                lease: net.dhcp_lease,
                dns: if net.dhcp_dns_enabled {
                    Some(vec![net.dhcp_dns_1, net.dhcp_dns_2].into_iter().flatten().collect())
                } else {
                    None
                },
            }),
            domain: net.domain_name,
            igmp_snooping: net.igmp_snooping,
            networkgroup: Some(net.networkgroup),
            extra: net.extra,
        })
        .collect()
}

fn export_firewall_groups(
    firewall_groups: Vec<FirewallGroup>,
) -> Vec<crate::config::schema::FirewallGroupConfig> {
    firewall_groups
        .into_iter()
        .map(|group| crate::config::schema::FirewallGroupConfig {
            name: group.name,
            group_type: group.group_type,
            members: group.group_members,
        })
        .collect()
}

fn export_firewall_config(
    firewall_rules: Vec<FirewallRule>,
) -> crate::config::schema::FirewallConfig {
    let mut firewall_config = crate::config::schema::FirewallConfig::default();
    for rule in firewall_rules {
        let ruleset = rule.ruleset;
        let entry = FirewallRuleConfig {
            name: rule.name,
            enabled: rule.enabled,
            rule_index: Some(rule.rule_index),
            action: rule.action,
            protocol: rule.protocol,
            logging: rule.logging,
            state_new: rule.state_new,
            state_established: rule.state_established,
            state_invalid: rule.state_invalid,
            state_related: rule.state_related,
            src_address: rule.src_address,
            src_network: rule.src_networkconf_id,
            src_firewallgroup: rule.src_firewallgroup_ids,
            src_mac_address: rule.src_mac_address,
            src_port: rule.src_port,
            dst_address: rule.dst_address,
            dst_network: rule.dst_networkconf_id,
            dst_firewallgroup: rule.dst_firewallgroup_ids,
            dst_port: rule.dst_port,
        };
        match ruleset {
            FirewallRuleset::WanIn => firewall_config.wan_in.push(entry),
            FirewallRuleset::WanOut => firewall_config.wan_out.push(entry),
            FirewallRuleset::WanLocal => firewall_config.wan_local.push(entry),
            FirewallRuleset::LanIn => firewall_config.lan_in.push(entry),
            FirewallRuleset::LanOut => firewall_config.lan_out.push(entry),
            FirewallRuleset::LanLocal => firewall_config.lan_local.push(entry),
            FirewallRuleset::GuestIn => firewall_config.guest_in.push(entry),
            FirewallRuleset::GuestOut => firewall_config.guest_out.push(entry),
            FirewallRuleset::GuestLocal => firewall_config.guest_local.push(entry),
            FirewallRuleset::InterVlan => firewall_config.inter_vlan.push(entry),
        }
    }
    firewall_config
}

fn export_port_forwards(
    port_forwards: Vec<PortForward>,
) -> Vec<crate::config::schema::PortForwardConfig> {
    port_forwards
        .into_iter()
        .map(|pf| crate::config::schema::PortForwardConfig {
            name: pf.name,
            enabled: pf.enabled,
            src: pf.src.map(|src| crate::config::schema::PortForwardSourceConfig::Limited {
                src_ip: Some(src),
                src_firewallgroup: None,
            }),
            dst_port: pf.dst_port,
            fwd: pf.forward_ip,
            fwd_port: pf.forward_port,
            proto: pf.protocol,
            log: pf.log,
            pfwd_interface: Some(pf.interface),
        })
        .collect()
}

fn export_traffic_rules(
    traffic_rules: Vec<TrafficRule>,
) -> Vec<crate::config::schema::TrafficRuleConfig> {
    traffic_rules
        .into_iter()
        .map(|rule| {
            let schedule = if rule.schedule_days.is_empty() {
                None
            } else {
                Some(crate::config::schema::ScheduleConfig {
                    days: rule.schedule_days,
                    start_time: rule.schedule_start.unwrap_or_else(|| "00:00".to_string()),
                    end_time: rule.schedule_end.unwrap_or_else(|| "23:59".to_string()),
                })
            };
            crate::config::schema::TrafficRuleConfig {
                description: rule.description,
                enabled: rule.enabled,
                action: match rule.action.as_str() {
                    "ALLOW" => crate::config::schema::TrafficActionConfig::Allow,
                    "RATE_LIMIT" => crate::config::schema::TrafficActionConfig::Limit,
                    _ => crate::config::schema::TrafficActionConfig::Block,
                },
                matching_target: crate::config::schema::MatchingTargetConfig::AllTraffic,
                target_devices: crate::config::schema::TargetDevicesConfig::AllClients,
                schedule,
                bandwidth_limit: rule.bandwidth_limit.map(|limit| {
                    crate::config::schema::BandwidthLimitConfig {
                        download_kbps: limit.download_kbps,
                        upload_kbps: limit.upload_kbps,
                    }
                }),
            }
        })
        .collect()
}

fn export_vpn(
    wireguard_servers: &[WireGuardServer],
    wireguard_peers: &[WireGuardPeer],
    site_vpns: Vec<SiteVpn>,
) -> crate::config::schema::VpnConfig {
    let wireguard = wireguard_servers.first().map(|server| {
        let allowed_ips = wireguard_peers
            .iter()
            .map(|peer| crate::config::schema::WireGuardClientConfig {
                name: peer.name.clone(),
                public_key: peer.public_key.clone(),
                preshared_key: peer.preshared_key.clone(),
                assigned_ip: peer.allowed_ips.first().cloned(),
                allowed_ips: peer.allowed_ips.clone(),
                enabled: peer.enabled,
            })
            .collect();
        crate::config::schema::WireGuardConfig {
            name: Some(server.name.clone()),
            enabled: server.enabled,
            port: Some(server.port),
            network: server.address.clone(),
            dns: server.dns_servers.clone(),
            allowed_networks: server.allowed_networks.clone(),
            clients: allowed_ips,
        }
    });

    crate::config::schema::VpnConfig {
        wireguard,
        site_to_site: site_vpns
            .into_iter()
            .map(|vpn| crate::config::schema::SiteToSiteConfig {
                name: vpn.name,
                enabled: vpn.enabled,
                vpn_type: vpn.vpn_type,
                remote_ip: vpn.remote_host,
                remote_subnets: vpn.remote_subnets,
                local_subnets: vpn.local_subnets,
                psk: vpn.psk,
            })
            .collect(),
    }
}

fn export_dhcp_reservations(
    dhcp_reservations: Vec<DhcpReservation>,
) -> Vec<crate::config::schema::DhcpReservationConfig> {
    dhcp_reservations
        .into_iter()
        .map(|reservation| crate::config::schema::DhcpReservationConfig {
            mac: reservation.mac_address,
            ip: reservation.ip_address,
            name: reservation.name,
            network: reservation.network_id,
        })
        .collect()
}

fn export_clients(client_groups: Vec<ClientGroup>, blocked_clients: Vec<Client>) -> ClientsConfig {
    ClientsConfig {
        blocked: blocked_clients
            .into_iter()
            .map(|client| crate::config::schema::BlockedClientConfig {
                mac: client.mac_address,
                note: client.note,
            })
            .collect(),
        groups: client_groups
            .into_iter()
            .map(|group| crate::config::schema::ClientGroupConfig {
                name: group.name,
                qos_rate_max_down: group.qos_rate_max_down,
                qos_rate_max_up: group.qos_rate_max_up,
            })
            .collect(),
    }
}

async fn export_dns_config(
    dns_service: &DnsService<'_>,
    dns_records: Vec<DnsRecord>,
) -> Option<DnsConfig> {
    dns_service.get_settings().await.map_or(None, move |settings| {
        let upstream = settings
            .get("upstream_dns")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let filtering = settings.get("filtering_enabled").and_then(|value| value.as_bool());
        let level = settings
            .get("filtering_level")
            .and_then(|value| value.as_str())
            .and_then(|value| serde_json::from_str(&format!("\"{value}\"")).ok())
            .unwrap_or(crate::types::DnsFilterLevel::None);
        Some(DnsConfig {
            records: dns_records
                .into_iter()
                .map(|record| crate::config::schema::DnsRecordConfig {
                    key: record.key,
                    value: record.value,
                    record_type: record.record_type,
                    ttl: record.ttl,
                })
                .collect(),
            upstream,
            filtering: filtering
                .map(|enabled| crate::config::schema::DnsFilteringConfig { enabled, level }),
        })
    })
}

/// Export current configuration into a declarative config object.
pub async fn export_config(client: &UnifiClient) -> Result<UnifiDeclarativeConfig> {
    let network_service = NetworkService::new(client);
    let firewall_service = FirewallService::new(client);
    let port_forward_service = PortForwardService::new(client);
    let traffic_service = TrafficService::new(client);
    let vpn_service = VpnService::new(client);
    let dhcp_service = DhcpService::new(client);
    let dns_service = DnsService::new(client);
    let client_service = ClientService::new(client);

    let networks = network_service.list().await?;
    let firewall_groups = firewall_service.list_groups().await?;
    let firewall_rules = firewall_service.list_rules().await?;
    let port_forwards = port_forward_service.list().await?;
    let traffic_rules = traffic_service.list().await?;
    let wireguard_servers = vpn_service.list_wireguard_servers().await?;
    let wireguard_peers = vpn_service.list_wireguard_peers().await?;
    let site_vpns = vpn_service.list_site_vpns().await?;
    let dhcp_reservations = dhcp_service.list_reservations().await?;
    let dns_records = dns_service.list().await?;
    let client_groups = client_service.list_groups().await?;
    let blocked_clients = client_service.list_blocked().await?;

    let firewall_config = export_firewall_config(firewall_rules);
    let dns_settings = export_dns_config(&dns_service, dns_records).await;
    let clients = export_clients(client_groups, blocked_clients);

    Ok(UnifiDeclarativeConfig {
        unifi: export_unifi_config(client),
        networks: export_networks(networks),
        firewall_groups: export_firewall_groups(firewall_groups),
        firewall: firewall_config,
        port_forward: export_port_forwards(port_forwards),
        traffic_rules: export_traffic_rules(traffic_rules),
        vpn: Some(export_vpn(&wireguard_servers, &wireguard_peers, site_vpns)),
        dhcp_reservations: export_dhcp_reservations(dhcp_reservations),
        dns: dns_settings,
        clients: Some(clients),
    })
}
