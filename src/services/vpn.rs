//! VPN management service.

use tracing::instrument;

use crate::client::UnifiClient;
use crate::error::{Result, UnifiError};
use crate::models::vpn::{SiteVpn, WireGuardPeer, WireGuardServer};

/// Service for managing VPN configurations.
pub struct VpnService<'a> {
    client: &'a UnifiClient,
}

impl<'a> VpnService<'a> {
    /// Create a new VPN service.
    #[must_use]
    pub const fn new(client: &'a UnifiClient) -> Self {
        Self { client }
    }

    /// List WireGuard servers.
    #[instrument(skip(self))]
    pub async fn list_wireguard_servers(&self) -> Result<Vec<WireGuardServer>> {
        self.client.get("rest/wg/server").await
    }

    /// Get a WireGuard server by ID.
    #[instrument(skip(self))]
    pub async fn get_wireguard_server(&self, id: &str) -> Result<WireGuardServer> {
        let servers: Vec<WireGuardServer> =
            self.client.get(&format!("rest/wg/server/{id}")).await?;
        servers
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::NotFound(format!("WireGuard server {id} not found")))
    }

    /// Create a WireGuard server.
    #[instrument(skip(self, server))]
    pub async fn create_wireguard_server(
        &self,
        server: &WireGuardServer,
    ) -> Result<WireGuardServer> {
        let servers: Vec<WireGuardServer> = self.client.post("rest/wg/server", server).await?;
        servers
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No server returned".to_string()))
    }

    /// Update a WireGuard server.
    #[instrument(skip(self, server))]
    pub async fn update_wireguard_server(
        &self,
        id: &str,
        server: &WireGuardServer,
    ) -> Result<WireGuardServer> {
        let servers: Vec<WireGuardServer> =
            self.client.put(&format!("rest/wg/server/{id}"), server).await?;
        servers
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No server returned".to_string()))
    }

    /// Delete a WireGuard server.
    #[instrument(skip(self))]
    pub async fn delete_wireguard_server(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("rest/wg/server/{id}")).await
    }

    /// List WireGuard peers.
    #[instrument(skip(self))]
    pub async fn list_wireguard_peers(&self) -> Result<Vec<WireGuardPeer>> {
        self.client.get("rest/wg/peer").await
    }

    /// List peers for a specific server.
    #[instrument(skip(self))]
    pub async fn list_wireguard_peers_for_server(
        &self,
        server_id: &str,
    ) -> Result<Vec<WireGuardPeer>> {
        let peers = self.list_wireguard_peers().await?;
        Ok(peers.into_iter().filter(|p| p.server_id.as_deref() == Some(server_id)).collect())
    }

    /// Get a WireGuard peer by ID.
    #[instrument(skip(self))]
    pub async fn get_wireguard_peer(&self, id: &str) -> Result<WireGuardPeer> {
        let peers: Vec<WireGuardPeer> = self.client.get(&format!("rest/wg/peer/{id}")).await?;
        peers
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::NotFound(format!("WireGuard peer {id} not found")))
    }

    /// Create a WireGuard peer.
    #[instrument(skip(self, peer))]
    pub async fn create_wireguard_peer(&self, peer: &WireGuardPeer) -> Result<WireGuardPeer> {
        let peers: Vec<WireGuardPeer> = self.client.post("rest/wg/peer", peer).await?;
        peers
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No peer returned".to_string()))
    }

    /// Update a WireGuard peer.
    #[instrument(skip(self, peer))]
    pub async fn update_wireguard_peer(
        &self,
        id: &str,
        peer: &WireGuardPeer,
    ) -> Result<WireGuardPeer> {
        let peers: Vec<WireGuardPeer> =
            self.client.put(&format!("rest/wg/peer/{id}"), peer).await?;
        peers
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No peer returned".to_string()))
    }

    /// Delete a WireGuard peer.
    #[instrument(skip(self))]
    pub async fn delete_wireguard_peer(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("rest/wg/peer/{id}")).await
    }

    /// Enable or disable a WireGuard peer.
    #[instrument(skip(self))]
    pub async fn set_wireguard_peer_enabled(
        &self,
        id: &str,
        enabled: bool,
    ) -> Result<WireGuardPeer> {
        let mut peer = self.get_wireguard_peer(id).await?;
        peer.enabled = enabled;
        self.update_wireguard_peer(id, &peer).await
    }

    /// List site-to-site VPNs.
    #[instrument(skip(self))]
    pub async fn list_site_vpns(&self) -> Result<Vec<SiteVpn>> {
        self.client.get("rest/vpn").await
    }

    /// Get a site-to-site VPN by ID.
    #[instrument(skip(self))]
    pub async fn get_site_vpn(&self, id: &str) -> Result<SiteVpn> {
        let vpns: Vec<SiteVpn> = self.client.get(&format!("rest/vpn/{id}")).await?;
        vpns.into_iter()
            .next()
            .ok_or_else(|| UnifiError::NotFound(format!("Site VPN {id} not found")))
    }

    /// Create a site-to-site VPN.
    #[instrument(skip(self, vpn))]
    pub async fn create_site_vpn(&self, vpn: &SiteVpn) -> Result<SiteVpn> {
        let vpns: Vec<SiteVpn> = self.client.post("rest/vpn", vpn).await?;
        vpns.into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No VPN returned".to_string()))
    }

    /// Update a site-to-site VPN.
    #[instrument(skip(self, vpn))]
    pub async fn update_site_vpn(&self, id: &str, vpn: &SiteVpn) -> Result<SiteVpn> {
        let vpns: Vec<SiteVpn> = self.client.put(&format!("rest/vpn/{id}"), vpn).await?;
        vpns.into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No VPN returned".to_string()))
    }

    /// Delete a site-to-site VPN.
    #[instrument(skip(self))]
    pub async fn delete_site_vpn(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("rest/vpn/{id}")).await
    }

    /// Enable or disable a site-to-site VPN.
    #[instrument(skip(self))]
    pub async fn set_site_vpn_enabled(&self, id: &str, enabled: bool) -> Result<SiteVpn> {
        let mut vpn = self.get_site_vpn(id).await?;
        vpn.enabled = enabled;
        self.update_site_vpn(id, &vpn).await
    }

    /// Get VPN status summary.
    #[instrument(skip(self))]
    pub async fn get_status(&self) -> Result<Vec<serde_json::Value>> {
        self.client.get("stat/vpn").await
    }

    /// Generate WireGuard client configuration for a peer.
    #[instrument(skip(self))]
    pub async fn generate_wireguard_peer_config(&self, id: &str) -> Result<String> {
        let candidates = [
            format!("rest/wg/peer/{id}/config"),
            format!("rest/wg/peer/{id}/download"),
            format!("rest/wg/peer/{id}/configfile"),
        ];

        let mut last_err = None;
        for path in candidates {
            match self.client.get_raw(&path).await {
                Ok(config) => return Ok(config),
                Err(err) => last_err = Some(err),
            }
        }

        Err(last_err.unwrap_or_else(|| {
            UnifiError::NotFound(format!("WireGuard peer {id} config not found"))
        }))
    }
}
