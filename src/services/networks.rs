//! Network and VLAN management service.

use tracing::instrument;

use crate::client::UnifiClient;
use crate::error::Result;
use crate::models::Network;

/// Service for managing networks and VLANs.
pub struct NetworkService<'a> {
    client: &'a UnifiClient,
}

impl<'a> NetworkService<'a> {
    /// Create a new network service.
    #[must_use]
    pub const fn new(client: &'a UnifiClient) -> Self {
        Self { client }
    }

    /// List all networks/VLANs.
    #[instrument(skip(self))]
    pub async fn list(&self) -> Result<Vec<Network>> {
        self.client.get("rest/networkconf").await
    }

    /// Get a network by ID.
    #[instrument(skip(self))]
    pub async fn get(&self, id: &str) -> Result<Network> {
        let networks: Vec<Network> = self.client.get(&format!("rest/networkconf/{id}")).await?;
        networks
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::UnifiError::NotFound(format!("Network {id} not found")))
    }

    /// Get a network by name.
    #[instrument(skip(self))]
    pub async fn get_by_name(&self, name: &str) -> Result<Option<Network>> {
        let networks = self.list().await?;
        Ok(networks.into_iter().find(|n| n.name == name))
    }

    /// Create a new network/VLAN.
    #[instrument(skip(self, network))]
    pub async fn create(&self, network: &Network) -> Result<Network> {
        let networks: Vec<Network> = self.client.post("rest/networkconf", network).await?;
        networks.into_iter().next().ok_or_else(|| {
            crate::error::UnifiError::InvalidResponse("No network returned".to_string())
        })
    }

    /// Update an existing network.
    #[instrument(skip(self, network))]
    pub async fn update(&self, id: &str, network: &Network) -> Result<Network> {
        let networks: Vec<Network> =
            self.client.put(&format!("rest/networkconf/{id}"), network).await?;
        networks.into_iter().next().ok_or_else(|| {
            crate::error::UnifiError::InvalidResponse("No network returned".to_string())
        })
    }

    /// Delete a network.
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("rest/networkconf/{id}")).await
    }

    /// Create a corporate network with default settings.
    #[instrument(skip(self))]
    pub async fn create_corporate(
        &self,
        name: &str,
        subnet: &str,
        dhcp_start: Option<&str>,
        dhcp_stop: Option<&str>,
    ) -> Result<Network> {
        let mut network = Network::new_corporate(name, subnet);
        if let (Some(start), Some(stop)) = (dhcp_start, dhcp_stop) {
            network = network.with_dhcp_range(start, stop);
        }
        self.create(&network).await
    }

    /// Create a VLAN network.
    #[instrument(skip(self))]
    pub async fn create_vlan(
        &self,
        name: &str,
        vlan_id: u16,
        subnet: &str,
        dhcp_start: Option<&str>,
        dhcp_stop: Option<&str>,
    ) -> Result<Network> {
        let mut network = Network::new_vlan(name, vlan_id, subnet);
        if let (Some(start), Some(stop)) = (dhcp_start, dhcp_stop) {
            network = network.with_dhcp_range(start, stop);
        }
        self.create(&network).await
    }

    /// Create a guest network.
    #[instrument(skip(self))]
    pub async fn create_guest(
        &self,
        name: &str,
        vlan_id: u16,
        subnet: &str,
        dhcp_start: Option<&str>,
        dhcp_stop: Option<&str>,
    ) -> Result<Network> {
        let mut network = Network::new_guest(name, vlan_id, subnet);
        if let (Some(start), Some(stop)) = (dhcp_start, dhcp_stop) {
            network = network.with_dhcp_range(start, stop);
        }
        self.create(&network).await
    }

    /// Enable or disable a network.
    #[instrument(skip(self))]
    pub async fn set_enabled(&self, id: &str, enabled: bool) -> Result<Network> {
        let mut network = self.get(id).await?;
        network.enabled = enabled;
        self.update(id, &network).await
    }

    /// Update DHCP settings for a network.
    #[instrument(skip(self))]
    pub async fn update_dhcp(
        &self,
        id: &str,
        enabled: bool,
        start: Option<&str>,
        stop: Option<&str>,
        lease_time: Option<u32>,
    ) -> Result<Network> {
        let mut network = self.get(id).await?;
        network.dhcp_enabled = enabled;
        if let Some(s) = start {
            network.dhcp_start = Some(s.to_string());
        }
        if let Some(s) = stop {
            network.dhcp_stop = Some(s.to_string());
        }
        if let Some(lease) = lease_time {
            network.dhcp_lease = Some(lease);
        }
        self.update(id, &network).await
    }

    /// Set DNS servers for a network's DHCP.
    #[instrument(skip(self))]
    pub async fn set_dns_servers(
        &self,
        id: &str,
        dns1: &str,
        dns2: Option<&str>,
    ) -> Result<Network> {
        let mut network = self.get(id).await?;
        network.dhcp_dns_enabled = true;
        network.dhcp_dns_1 = Some(dns1.to_string());
        network.dhcp_dns_2 = dns2.map(String::from);
        self.update(id, &network).await
    }

    /// Set a controller-specific field (e.g., isolation settings).
    #[instrument(skip(self))]
    pub async fn set_custom_field(
        &self,
        id: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<Network> {
        let mut network = self.get(id).await?;
        network.extra.insert(key.to_string(), value);
        self.update(id, &network).await
    }
}
