//! Port forwarding management service.

use tracing::instrument;

use crate::client::UnifiClient;
use crate::error::{Result, UnifiError};
use crate::models::PortForward;
use crate::types::Protocol;

/// Service for managing port forwarding rules.
pub struct PortForwardService<'a> {
    client: &'a UnifiClient,
}

impl<'a> PortForwardService<'a> {
    /// Create a new port forward service.
    #[must_use]
    pub const fn new(client: &'a UnifiClient) -> Self {
        Self { client }
    }

    /// List all port forwarding rules.
    #[instrument(skip(self))]
    pub async fn list(&self) -> Result<Vec<PortForward>> {
        self.client.get("rest/portforward").await
    }

    /// Get a port forward rule by ID.
    #[instrument(skip(self))]
    pub async fn get(&self, id: &str) -> Result<PortForward> {
        let rules: Vec<PortForward> = self.client.get(&format!("rest/portforward/{id}")).await?;
        rules
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::NotFound(format!("Port forward {id} not found")))
    }

    /// Get a port forward rule by name.
    #[instrument(skip(self))]
    pub async fn get_by_name(&self, name: &str) -> Result<Option<PortForward>> {
        let rules = self.list().await?;
        Ok(rules.into_iter().find(|r| r.name == name))
    }

    /// Create a port forwarding rule.
    #[instrument(skip(self, rule))]
    pub async fn create(&self, rule: &PortForward) -> Result<PortForward> {
        let rules: Vec<PortForward> = self.client.post("rest/portforward", rule).await?;
        rules
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No rule returned".to_string()))
    }

    /// Update a port forwarding rule.
    #[instrument(skip(self, rule))]
    pub async fn update(&self, id: &str, rule: &PortForward) -> Result<PortForward> {
        let rules: Vec<PortForward> =
            self.client.put(&format!("rest/portforward/{id}"), rule).await?;
        rules
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No rule returned".to_string()))
    }

    /// Delete a port forwarding rule.
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("rest/portforward/{id}")).await
    }

    /// Enable or disable a port forward rule.
    #[instrument(skip(self))]
    pub async fn set_enabled(&self, id: &str, enabled: bool) -> Result<PortForward> {
        let mut rule = self.get(id).await?;
        rule.enabled = enabled;
        self.update(id, &rule).await
    }

    /// Create a TCP port forward.
    #[instrument(skip(self))]
    pub async fn create_tcp(
        &self,
        name: &str,
        wan_port: &str,
        internal_ip: &str,
        internal_port: &str,
    ) -> Result<PortForward> {
        let rule = PortForward::tcp(name, wan_port, internal_ip, internal_port);
        self.create(&rule).await
    }

    /// Create a UDP port forward.
    #[instrument(skip(self))]
    pub async fn create_udp(
        &self,
        name: &str,
        wan_port: &str,
        internal_ip: &str,
        internal_port: &str,
    ) -> Result<PortForward> {
        let rule = PortForward::udp(name, wan_port, internal_ip, internal_port);
        self.create(&rule).await
    }

    /// Create a TCP+UDP port forward.
    #[instrument(skip(self))]
    pub async fn create_tcp_udp(
        &self,
        name: &str,
        wan_port: &str,
        internal_ip: &str,
        internal_port: &str,
    ) -> Result<PortForward> {
        let rule = PortForward::tcp_udp(name, wan_port, internal_ip, internal_port);
        self.create(&rule).await
    }

    /// Create a port forward with source restriction.
    #[instrument(skip(self))]
    pub async fn create_restricted(
        &self,
        name: &str,
        protocol: Protocol,
        wan_port: &str,
        internal_ip: &str,
        internal_port: &str,
        allowed_source: &str,
    ) -> Result<PortForward> {
        let rule = PortForward::new(name, wan_port, internal_ip, internal_port)
            .with_protocol(protocol)
            .with_source(allowed_source);
        self.create(&rule).await
    }

    /// Update the forward destination for a rule.
    #[instrument(skip(self))]
    pub async fn update_destination(
        &self,
        id: &str,
        internal_ip: &str,
        internal_port: &str,
    ) -> Result<PortForward> {
        let mut rule = self.get(id).await?;
        rule.forward_ip = internal_ip.to_string();
        rule.forward_port = internal_port.to_string();
        self.update(id, &rule).await
    }

    /// Update the WAN port for a rule.
    #[instrument(skip(self))]
    pub async fn update_wan_port(&self, id: &str, wan_port: &str) -> Result<PortForward> {
        let mut rule = self.get(id).await?;
        rule.dst_port = wan_port.to_string();
        self.update(id, &rule).await
    }
}
