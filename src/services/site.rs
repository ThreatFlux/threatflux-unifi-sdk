//! Site management service.

use tracing::instrument;

use crate::client::UnifiClient;
use crate::error::{Result, UnifiError};
use crate::models::site::{Site, SiteStats, SystemInfo};

/// Service for managing sites and getting statistics.
pub struct SiteService<'a> {
    client: &'a UnifiClient,
}

impl<'a> SiteService<'a> {
    /// Create a new site service.
    #[must_use]
    pub const fn new(client: &'a UnifiClient) -> Self {
        Self { client }
    }

    /// List all sites accessible by the current user.
    /// Note: This uses the stat/sites endpoint which returns site info for the current site.
    /// For multi-site listing, the controller API at /api/self/sites would be needed.
    #[instrument(skip(self))]
    pub async fn list(&self) -> Result<Vec<Site>> {
        self.client.get("stat/sites").await
    }

    /// Get the current site.
    #[instrument(skip(self))]
    pub async fn get_current(&self) -> Result<Site> {
        let sites = self.list().await?;
        let current_site = self.client.site();
        sites
            .into_iter()
            .find(|s| s.name == current_site)
            .ok_or_else(|| UnifiError::NotFound(format!("Site {current_site} not found")))
    }

    /// Get a site by name.
    #[instrument(skip(self))]
    pub async fn get_by_name(&self, name: &str) -> Result<Option<Site>> {
        let sites = self.list().await?;
        Ok(sites.into_iter().find(|s| s.name == name))
    }

    /// Get site statistics.
    #[instrument(skip(self))]
    pub async fn get_stats(&self) -> Result<SiteStats> {
        let stats: Vec<SiteStats> = self.client.get("stat/sites").await?;
        stats
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No stats returned".to_string()))
    }

    /// Get system information.
    #[instrument(skip(self))]
    pub async fn get_system_info(&self) -> Result<SystemInfo> {
        let info: Vec<SystemInfo> = self.client.get("stat/sysinfo").await?;
        info.into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No system info returned".to_string()))
    }

    /// Get site health.
    #[instrument(skip(self))]
    pub async fn get_health(&self) -> Result<Site> {
        let sites: Vec<Site> = self.client.get("stat/health").await?;
        sites
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No health data returned".to_string()))
    }

    /// Create a new site.
    #[instrument(skip(self))]
    pub async fn create(&self, name: &str, description: &str) -> Result<Site> {
        let payload = serde_json::json!({
            "cmd": "add-site",
            "name": name,
            "desc": description
        });
        let sites: Vec<Site> = self.client.command("sitemgr", &payload).await?;
        sites
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No site returned".to_string()))
    }

    /// Update site description.
    #[instrument(skip(self))]
    pub async fn update_description(&self, site_id: &str, description: &str) -> Result<Site> {
        let payload = serde_json::json!({
            "cmd": "update-site",
            "desc": description
        });
        let sites: Vec<Site> = self.client.command("sitemgr", &payload).await?;
        sites
            .into_iter()
            .find(|s| s.id.as_deref() == Some(site_id))
            .ok_or_else(|| UnifiError::NotFound(format!("Site {site_id} not found")))
    }

    /// Delete a site.
    #[instrument(skip(self))]
    pub async fn delete(&self, site_name: &str) -> Result<()> {
        let payload = serde_json::json!({
            "cmd": "delete-site",
            "site": site_name
        });
        let _: serde_json::Value = self.client.command("sitemgr", &payload).await?;
        Ok(())
    }

    /// Get events for the site.
    #[instrument(skip(self))]
    pub async fn get_events(&self, limit: Option<u32>) -> Result<Vec<serde_json::Value>> {
        let limit = limit.unwrap_or(100);
        self.client.get(&format!("stat/event?_limit={limit}")).await
    }

    /// Get alarms for the site.
    #[instrument(skip(self))]
    pub async fn get_alarms(&self) -> Result<Vec<serde_json::Value>> {
        self.client.get("stat/alarm").await
    }

    /// Archive all alarms.
    #[instrument(skip(self))]
    pub async fn archive_alarms(&self) -> Result<()> {
        let payload = serde_json::json!({
            "cmd": "archive-all-alarms"
        });
        let _: serde_json::Value = self.client.command("evtmgr", &payload).await?;
        Ok(())
    }

    /// Get DPI (Deep Packet Inspection) stats.
    #[instrument(skip(self))]
    pub async fn get_dpi_stats(&self) -> Result<Vec<serde_json::Value>> {
        self.client.get("stat/sitedpi").await
    }

    /// Force provision all devices.
    #[instrument(skip(self))]
    pub async fn force_provision_all(&self) -> Result<()> {
        let payload = serde_json::json!({
            "cmd": "force-provision"
        });
        let _: serde_json::Value = self.client.command("devmgr", &payload).await?;
        Ok(())
    }
}
