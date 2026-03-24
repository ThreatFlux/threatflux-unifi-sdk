//! Device management service.

use tracing::instrument;

use crate::client::UnifiClient;
use crate::error::{Result, UnifiError};
use crate::models::device::{Device, PortOverride};
use crate::types::{DeviceType, PoeMode};

/// Service for managing UniFi devices.
pub struct DeviceService<'a> {
    client: &'a UnifiClient,
}

impl<'a> DeviceService<'a> {
    /// Create a new device service.
    #[must_use]
    pub const fn new(client: &'a UnifiClient) -> Self {
        Self { client }
    }

    /// List all devices.
    #[instrument(skip(self))]
    pub async fn list(&self) -> Result<Vec<Device>> {
        self.client.get("stat/device").await
    }

    /// List devices by type.
    #[instrument(skip(self))]
    pub async fn list_by_type(&self, device_type: DeviceType) -> Result<Vec<Device>> {
        let devices = self.list().await?;
        Ok(devices.into_iter().filter(|d| d.device_type == device_type).collect())
    }

    /// List access points.
    #[instrument(skip(self))]
    pub async fn list_aps(&self) -> Result<Vec<Device>> {
        self.list_by_type(DeviceType::Uap).await
    }

    /// List switches.
    #[instrument(skip(self))]
    pub async fn list_switches(&self) -> Result<Vec<Device>> {
        self.list_by_type(DeviceType::Usw).await
    }

    /// List gateways.
    #[instrument(skip(self))]
    pub async fn list_gateways(&self) -> Result<Vec<Device>> {
        let devices = self.list().await?;
        Ok(devices.into_iter().filter(|d| d.is_gateway()).collect())
    }

    /// Get a device by MAC address.
    #[instrument(skip(self))]
    pub async fn get(&self, mac: &str) -> Result<Device> {
        let devices: Vec<Device> = self.client.get(&format!("stat/device/{mac}")).await?;
        devices
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::NotFound(format!("Device {mac} not found")))
    }

    /// Get device by name.
    #[instrument(skip(self))]
    pub async fn get_by_name(&self, name: &str) -> Result<Option<Device>> {
        let devices = self.list().await?;
        Ok(devices.into_iter().find(|d| d.name.as_deref() == Some(name)))
    }

    /// Adopt a device.
    #[instrument(skip(self))]
    pub async fn adopt(&self, mac: &str) -> Result<()> {
        let cmd = serde_json::json!({
            "cmd": "adopt",
            "mac": mac
        });
        let _: serde_json::Value = self.client.command("devmgr", &cmd).await?;
        Ok(())
    }

    /// Forget a device.
    #[instrument(skip(self))]
    pub async fn forget(&self, mac: &str) -> Result<()> {
        let cmd = serde_json::json!({
            "cmd": "forget",
            "mac": mac
        });
        let _: serde_json::Value = self.client.command("devmgr", &cmd).await?;
        Ok(())
    }

    /// Restart a device.
    #[instrument(skip(self))]
    pub async fn restart(&self, mac: &str) -> Result<()> {
        let cmd = serde_json::json!({
            "cmd": "restart",
            "mac": mac
        });
        let _: serde_json::Value = self.client.command("devmgr", &cmd).await?;
        Ok(())
    }

    /// Force provision a device.
    #[instrument(skip(self))]
    pub async fn provision(&self, mac: &str) -> Result<()> {
        let cmd = serde_json::json!({
            "cmd": "force-provision",
            "mac": mac
        });
        let _: serde_json::Value = self.client.command("devmgr", &cmd).await?;
        Ok(())
    }

    /// Upgrade device firmware.
    #[instrument(skip(self))]
    pub async fn upgrade(&self, mac: &str) -> Result<()> {
        let cmd = serde_json::json!({
            "cmd": "upgrade",
            "mac": mac
        });
        let _: serde_json::Value = self.client.command("devmgr", &cmd).await?;
        Ok(())
    }

    /// Upgrade device to specific firmware.
    #[instrument(skip(self))]
    pub async fn upgrade_to(&self, mac: &str, firmware_url: &str) -> Result<()> {
        let cmd = serde_json::json!({
            "cmd": "upgrade-external",
            "mac": mac,
            "url": firmware_url
        });
        let _: serde_json::Value = self.client.command("devmgr", &cmd).await?;
        Ok(())
    }

    /// Set device name.
    #[instrument(skip(self))]
    pub async fn set_name(&self, mac: &str, name: &str) -> Result<()> {
        let device = self.get(mac).await?;
        if let Some(id) = device.id {
            let payload = serde_json::json!({ "name": name });
            let _: serde_json::Value =
                self.client.put(&format!("rest/device/{id}"), &payload).await?;
        }
        Ok(())
    }

    /// Enable or disable a device.
    #[instrument(skip(self))]
    pub async fn set_disabled(&self, mac: &str, disabled: bool) -> Result<()> {
        let device = self.get(mac).await?;
        if let Some(id) = device.id {
            let payload = serde_json::json!({ "disabled": disabled });
            let _: serde_json::Value =
                self.client.put(&format!("rest/device/{id}"), &payload).await?;
        }
        Ok(())
    }

    /// Set LED override.
    #[instrument(skip(self))]
    pub async fn set_led_override(&self, mac: &str, mode: &str) -> Result<()> {
        let device = self.get(mac).await?;
        if let Some(id) = device.id {
            let payload = serde_json::json!({ "led_override": mode });
            let _: serde_json::Value =
                self.client.put(&format!("rest/device/{id}"), &payload).await?;
        }
        Ok(())
    }

    /// Locate device (blink LEDs).
    #[instrument(skip(self))]
    pub async fn locate(&self, mac: &str, enabled: bool) -> Result<()> {
        let cmd = if enabled { "set-locate" } else { "unset-locate" };
        let payload = serde_json::json!({
            "cmd": cmd,
            "mac": mac
        });
        let _: serde_json::Value = self.client.command("devmgr", &payload).await?;
        Ok(())
    }

    /// Configure a switch port (port profile).
    #[instrument(skip(self))]
    pub async fn configure_port(
        &self,
        mac: &str,
        port_idx: u32,
        port_profile_id: &str,
    ) -> Result<()> {
        self.update_port_override(mac, port_idx, Some(port_profile_id), None, None).await
    }

    /// Set PoE mode for a switch port.
    #[instrument(skip(self))]
    pub async fn set_poe_mode(&self, mac: &str, port_idx: u32, mode: PoeMode) -> Result<()> {
        self.update_port_override(mac, port_idx, None, Some(mode), None).await
    }

    /// Update or create a port override entry.
    #[instrument(skip(self))]
    async fn update_port_override(
        &self,
        mac: &str,
        port_idx: u32,
        port_profile_id: Option<&str>,
        poe_mode: Option<PoeMode>,
        name: Option<&str>,
    ) -> Result<()> {
        let device = self.get(mac).await?;
        let id = device
            .id
            .ok_or_else(|| UnifiError::InvalidResponse("Device ID missing".to_string()))?;
        let mut overrides = device.port_overrides;

        let entry = overrides.iter_mut().find(|o| o.port_idx == port_idx);
        if let Some(existing) = entry {
            if let Some(profile) = port_profile_id {
                existing.portconf_id = Some(profile.to_string());
            }
            if let Some(mode) = poe_mode {
                existing.poe_mode = Some(mode.to_string());
            }
            if let Some(name) = name {
                existing.name = Some(name.to_string());
            }
        } else {
            overrides.push(PortOverride {
                port_idx,
                name: name.map(str::to_string),
                portconf_id: port_profile_id.map(str::to_string),
                poe_mode: poe_mode.map(|m| m.to_string()),
                aggregate_num_ports: None,
            });
        }

        let payload = serde_json::json!({
            "port_overrides": overrides
        });
        let _: serde_json::Value = self.client.put(&format!("rest/device/{id}"), &payload).await?;
        Ok(())
    }

    /// List devices needing upgrade.
    #[instrument(skip(self))]
    pub async fn list_upgradable(&self) -> Result<Vec<Device>> {
        let devices = self.list().await?;
        Ok(devices.into_iter().filter(|d| d.upgradable).collect())
    }

    /// List offline devices.
    #[instrument(skip(self))]
    pub async fn list_offline(&self) -> Result<Vec<Device>> {
        let devices = self.list().await?;
        Ok(devices.into_iter().filter(|d| !d.is_online()).collect())
    }

    /// List online devices.
    #[instrument(skip(self))]
    pub async fn list_online(&self) -> Result<Vec<Device>> {
        let devices = self.list().await?;
        Ok(devices.into_iter().filter(|d| d.is_online()).collect())
    }
}
