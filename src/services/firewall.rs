//! Firewall management service.

use tracing::instrument;

use crate::client::UnifiClient;
use crate::error::{Result, UnifiError};
use crate::models::{FirewallGroup, FirewallRule};
use crate::types::FirewallRuleset;
use std::collections::HashMap;

/// Service for managing firewall rules and groups.
pub struct FirewallService<'a> {
    client: &'a UnifiClient,
}

impl<'a> FirewallService<'a> {
    /// Create a new firewall service.
    #[must_use]
    pub const fn new(client: &'a UnifiClient) -> Self {
        Self { client }
    }

    /// List all firewall rules.
    #[instrument(skip(self))]
    pub async fn list_rules(&self) -> Result<Vec<FirewallRule>> {
        self.client.get("rest/firewallrule").await
    }

    /// List firewall rules by ruleset.
    #[instrument(skip(self))]
    pub async fn list_rules_by_ruleset(
        &self,
        ruleset: FirewallRuleset,
    ) -> Result<Vec<FirewallRule>> {
        let rules = self.list_rules().await?;
        Ok(rules.into_iter().filter(|r| r.ruleset == ruleset).collect())
    }

    /// Get a firewall rule by ID.
    #[instrument(skip(self))]
    pub async fn get_rule(&self, id: &str) -> Result<FirewallRule> {
        let rules: Vec<FirewallRule> = self.client.get(&format!("rest/firewallrule/{id}")).await?;
        rules
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::NotFound(format!("Firewall rule {id} not found")))
    }

    /// Create a firewall rule.
    #[instrument(skip(self, rule))]
    pub async fn create_rule(&self, rule: &FirewallRule) -> Result<FirewallRule> {
        let rules: Vec<FirewallRule> = self.client.post("rest/firewallrule", rule).await?;
        rules
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No rule returned".to_string()))
    }

    /// Update a firewall rule.
    #[instrument(skip(self, rule))]
    pub async fn update_rule(&self, id: &str, rule: &FirewallRule) -> Result<FirewallRule> {
        let rules: Vec<FirewallRule> =
            self.client.put(&format!("rest/firewallrule/{id}"), rule).await?;
        rules
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No rule returned".to_string()))
    }

    /// Delete a firewall rule.
    #[instrument(skip(self))]
    pub async fn delete_rule(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("rest/firewallrule/{id}")).await
    }

    /// Reorder firewall rules within a ruleset by updating rule_index values.
    #[instrument(skip(self, rule_ids))]
    pub async fn reorder_rules(
        &self,
        ruleset: FirewallRuleset,
        rule_ids: &[String],
    ) -> Result<Vec<FirewallRule>> {
        const RULE_INDEX_BASE: i32 = 1000;
        const RULE_INDEX_STEP: i32 = 10;

        let existing = self.list_rules_by_ruleset(ruleset).await?;
        let mut by_id: HashMap<String, FirewallRule> =
            existing.into_iter().filter_map(|r| r.id.clone().map(|id| (id, r))).collect();

        let mut updated = Vec::with_capacity(rule_ids.len());
        for (idx, id) in rule_ids.iter().enumerate() {
            let mut rule = by_id
                .remove(id)
                .ok_or_else(|| UnifiError::NotFound(format!("Firewall rule {id} not found")))?;
            let idx_i32 = i32::try_from(idx).map_err(|_| {
                UnifiError::InvalidInput(format!("rule index {idx} exceeds i32 range"))
            })?;
            let offset = idx_i32
                .checked_mul(RULE_INDEX_STEP)
                .ok_or_else(|| UnifiError::InvalidInput("rule index overflow".to_string()))?;
            rule.rule_index = RULE_INDEX_BASE
                .checked_add(offset)
                .ok_or_else(|| UnifiError::InvalidInput("rule index overflow".to_string()))?;
            updated.push(self.update_rule(id, &rule).await?);
        }

        Ok(updated)
    }

    /// Enable or disable a firewall rule.
    #[instrument(skip(self))]
    pub async fn set_rule_enabled(&self, id: &str, enabled: bool) -> Result<FirewallRule> {
        let mut rule = self.get_rule(id).await?;
        rule.enabled = enabled;
        self.update_rule(id, &rule).await
    }

    /// List all firewall groups.
    #[instrument(skip(self))]
    pub async fn list_groups(&self) -> Result<Vec<FirewallGroup>> {
        self.client.get("rest/firewallgroup").await
    }

    /// Get a firewall group by ID.
    #[instrument(skip(self))]
    pub async fn get_group(&self, id: &str) -> Result<FirewallGroup> {
        let groups: Vec<FirewallGroup> =
            self.client.get(&format!("rest/firewallgroup/{id}")).await?;
        groups
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::NotFound(format!("Firewall group {id} not found")))
    }

    /// Get a firewall group by name.
    #[instrument(skip(self))]
    pub async fn get_group_by_name(&self, name: &str) -> Result<Option<FirewallGroup>> {
        let groups = self.list_groups().await?;
        Ok(groups.into_iter().find(|g| g.name == name))
    }

    /// Create a firewall group.
    #[instrument(skip(self, group))]
    pub async fn create_group(&self, group: &FirewallGroup) -> Result<FirewallGroup> {
        let groups: Vec<FirewallGroup> = self.client.post("rest/firewallgroup", group).await?;
        groups
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No group returned".to_string()))
    }

    /// Update a firewall group.
    #[instrument(skip(self, group))]
    pub async fn update_group(&self, id: &str, group: &FirewallGroup) -> Result<FirewallGroup> {
        let groups: Vec<FirewallGroup> =
            self.client.put(&format!("rest/firewallgroup/{id}"), group).await?;
        groups
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No group returned".to_string()))
    }

    /// Delete a firewall group.
    #[instrument(skip(self))]
    pub async fn delete_group(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("rest/firewallgroup/{id}")).await
    }

    /// Add members to a firewall group.
    #[instrument(skip(self))]
    pub async fn add_group_members(&self, id: &str, members: &[String]) -> Result<FirewallGroup> {
        let mut group = self.get_group(id).await?;
        for member in members {
            if !group.group_members.contains(member) {
                group.group_members.push(member.clone());
            }
        }
        self.update_group(id, &group).await
    }

    /// Remove members from a firewall group.
    #[instrument(skip(self))]
    pub async fn remove_group_members(
        &self,
        id: &str,
        members: &[String],
    ) -> Result<FirewallGroup> {
        let mut group = self.get_group(id).await?;
        group.group_members.retain(|m| !members.contains(m));
        self.update_group(id, &group).await
    }

    /// Create an address group.
    #[instrument(skip(self))]
    pub async fn create_address_group(
        &self,
        name: &str,
        addresses: Vec<String>,
    ) -> Result<FirewallGroup> {
        let group = FirewallGroup::address_group(name, addresses);
        self.create_group(&group).await
    }

    /// Create a port group.
    #[instrument(skip(self))]
    pub async fn create_port_group(&self, name: &str, ports: Vec<String>) -> Result<FirewallGroup> {
        let group = FirewallGroup::port_group(name, ports);
        self.create_group(&group).await
    }

    /// Quick helper: Create a drop rule for a source address.
    #[instrument(skip(self))]
    pub async fn block_ip(
        &self,
        name: &str,
        ip: &str,
        ruleset: FirewallRuleset,
    ) -> Result<FirewallRule> {
        let rule = FirewallRule::drop(name, ruleset).with_src_address(ip).with_logging(true);
        self.create_rule(&rule).await
    }

    /// Quick helper: Create an accept rule for established/related traffic.
    #[instrument(skip(self))]
    pub async fn allow_established(
        &self,
        ruleset: FirewallRuleset,
        index: i32,
    ) -> Result<FirewallRule> {
        let rule = FirewallRule::accept("Allow Established/Related", ruleset)
            .with_index(index)
            .with_state_established_related();
        self.create_rule(&rule).await
    }

    /// Quick helper: Create a drop rule for invalid packets.
    #[instrument(skip(self))]
    pub async fn drop_invalid(&self, ruleset: FirewallRuleset, index: i32) -> Result<FirewallRule> {
        let rule = FirewallRule::drop("Drop Invalid", ruleset)
            .with_index(index)
            .with_state_invalid()
            .with_logging(true);
        self.create_rule(&rule).await
    }
}
