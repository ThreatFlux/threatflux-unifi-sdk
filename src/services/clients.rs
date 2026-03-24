//! Client management service.

use tracing::instrument;

use crate::client::UnifiClient;
use crate::error::{Result, UnifiError};
use crate::models::client::{Client, ClientGroup, ClientStats};
use crate::types::Timeframe;

/// Service for managing connected clients.
pub struct ClientService<'a> {
    client: &'a UnifiClient,
}

impl<'a> ClientService<'a> {
    /// Create a new client service.
    #[must_use]
    pub const fn new(client: &'a UnifiClient) -> Self {
        Self { client }
    }

    /// List all known clients (online and offline).
    #[instrument(skip(self))]
    pub async fn list_all(&self) -> Result<Vec<Client>> {
        self.client.get("rest/user").await
    }

    /// List only online/active clients.
    #[instrument(skip(self))]
    pub async fn list_online(&self) -> Result<Vec<Client>> {
        self.client.get("stat/sta").await
    }

    /// Get a client by MAC address.
    #[instrument(skip(self))]
    pub async fn get(&self, mac: &str) -> Result<Client> {
        let clients: Vec<Client> = self.client.get(&format!("rest/user/{mac}")).await?;
        clients
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::NotFound(format!("Client {mac} not found")))
    }

    /// Get a client by name.
    #[instrument(skip(self))]
    pub async fn get_by_name(&self, name: &str) -> Result<Option<Client>> {
        let clients = self.list_all().await?;
        Ok(clients.into_iter().find(|c| c.name.as_deref() == Some(name)))
    }

    /// Block a client.
    #[instrument(skip(self))]
    pub async fn block(&self, mac: &str) -> Result<()> {
        let cmd = serde_json::json!({
            "cmd": "block-sta",
            "mac": mac
        });
        let _: serde_json::Value = self.client.command("stamgr", &cmd).await?;
        Ok(())
    }

    /// Unblock a client.
    #[instrument(skip(self))]
    pub async fn unblock(&self, mac: &str) -> Result<()> {
        let cmd = serde_json::json!({
            "cmd": "unblock-sta",
            "mac": mac
        });
        let _: serde_json::Value = self.client.command("stamgr", &cmd).await?;
        Ok(())
    }

    /// Kick/disconnect a client.
    #[instrument(skip(self))]
    pub async fn kick(&self, mac: &str) -> Result<()> {
        let cmd = serde_json::json!({
            "cmd": "kick-sta",
            "mac": mac
        });
        let _: serde_json::Value = self.client.command("stamgr", &cmd).await?;
        Ok(())
    }

    /// Authorize a guest client.
    #[instrument(skip(self))]
    pub async fn authorize_guest(&self, mac: &str, minutes: u32) -> Result<()> {
        let cmd = serde_json::json!({
            "cmd": "authorize-guest",
            "mac": mac,
            "minutes": minutes
        });
        let _: serde_json::Value = self.client.command("stamgr", &cmd).await?;
        Ok(())
    }

    /// Unauthorize a guest client.
    #[instrument(skip(self))]
    pub async fn unauthorize_guest(&self, mac: &str) -> Result<()> {
        let cmd = serde_json::json!({
            "cmd": "unauthorize-guest",
            "mac": mac
        });
        let _: serde_json::Value = self.client.command("stamgr", &cmd).await?;
        Ok(())
    }

    /// Set a fixed IP for a client.
    #[instrument(skip(self))]
    pub async fn set_fixed_ip(&self, mac: &str, ip: &str, network_id: &str) -> Result<()> {
        let payload = serde_json::json!({
            "mac": mac,
            "use_fixedip": true,
            "fixed_ip": ip,
            "network_id": network_id
        });
        let _: serde_json::Value = self.client.put(&format!("rest/user/{mac}"), &payload).await?;
        Ok(())
    }

    /// Remove fixed IP from a client.
    #[instrument(skip(self))]
    pub async fn remove_fixed_ip(&self, mac: &str) -> Result<()> {
        let payload = serde_json::json!({
            "mac": mac,
            "use_fixedip": false
        });
        let _: serde_json::Value = self.client.put(&format!("rest/user/{mac}"), &payload).await?;
        Ok(())
    }

    /// Set client name/alias.
    #[instrument(skip(self))]
    pub async fn set_name(&self, mac: &str, name: &str) -> Result<()> {
        let payload = serde_json::json!({
            "mac": mac,
            "name": name
        });
        let _: serde_json::Value = self.client.put(&format!("rest/user/{mac}"), &payload).await?;
        Ok(())
    }

    /// Set client note.
    #[instrument(skip(self))]
    pub async fn set_note(&self, mac: &str, note: &str) -> Result<()> {
        let payload = serde_json::json!({
            "mac": mac,
            "note": note,
            "noted": true
        });
        let _: serde_json::Value = self.client.put(&format!("rest/user/{mac}"), &payload).await?;
        Ok(())
    }

    /// Assign a client to a group.
    #[instrument(skip(self))]
    pub async fn assign_group(&self, mac: &str, group_id: &str) -> Result<()> {
        let payload = serde_json::json!({
            "mac": mac,
            "usergroup_id": group_id
        });
        let _: serde_json::Value = self.client.put(&format!("rest/user/{mac}"), &payload).await?;
        Ok(())
    }

    /// List blocked clients.
    #[instrument(skip(self))]
    pub async fn list_blocked(&self) -> Result<Vec<Client>> {
        let clients = self.list_all().await?;
        Ok(clients.into_iter().filter(|c| c.blocked).collect())
    }

    /// List guest clients.
    #[instrument(skip(self))]
    pub async fn list_guests(&self) -> Result<Vec<Client>> {
        let clients = self.list_online().await?;
        Ok(clients.into_iter().filter(|c| c.is_guest).collect())
    }

    /// List wireless clients.
    #[instrument(skip(self))]
    pub async fn list_wireless(&self) -> Result<Vec<Client>> {
        let clients = self.list_online().await?;
        Ok(clients.into_iter().filter(|c| c.is_wireless()).collect())
    }

    /// List wired clients.
    #[instrument(skip(self))]
    pub async fn list_wired(&self) -> Result<Vec<Client>> {
        let clients = self.list_online().await?;
        Ok(clients.into_iter().filter(|c| c.is_wired).collect())
    }

    /// List client groups.
    #[instrument(skip(self))]
    pub async fn list_groups(&self) -> Result<Vec<ClientGroup>> {
        self.client.get("rest/usergroup").await
    }

    /// Create a client group.
    #[instrument(skip(self, group))]
    pub async fn create_group(&self, group: &ClientGroup) -> Result<ClientGroup> {
        let groups: Vec<ClientGroup> = self.client.post("rest/usergroup", group).await?;
        groups
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No group returned".to_string()))
    }

    /// Update a client group.
    #[instrument(skip(self, group))]
    pub async fn update_group(&self, id: &str, group: &ClientGroup) -> Result<ClientGroup> {
        let groups: Vec<ClientGroup> =
            self.client.put(&format!("rest/usergroup/{id}"), group).await?;
        groups
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No group returned".to_string()))
    }

    /// Delete a client group.
    #[instrument(skip(self))]
    pub async fn delete_group(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("rest/usergroup/{id}")).await
    }

    /// Get client statistics for a timeframe.
    #[instrument(skip(self))]
    pub async fn get_stats(&self, mac: &str, timeframe: Timeframe) -> Result<Option<ClientStats>> {
        let params = [
            ("mac", mac.to_string()),
            ("start", timeframe.start_timestamp().to_string()),
            ("end", timeframe.end_timestamp().to_string()),
        ];
        let stats: Vec<ClientStats> = self.client.get_with_query("stat/user", &params).await?;
        Ok(stats.into_iter().next())
    }
}
