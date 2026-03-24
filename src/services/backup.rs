//! Backup and restore service.

use tracing::instrument;

use crate::client::UnifiClient;
use crate::error::{Result, UnifiError};
use crate::models::backup::{Backup, BackupSettings};

/// Service for backup and restore operations.
pub struct BackupService<'a> {
    client: &'a UnifiClient,
}

impl<'a> BackupService<'a> {
    /// Create a new backup service.
    #[must_use]
    pub const fn new(client: &'a UnifiClient) -> Self {
        Self { client }
    }

    /// List all available backups.
    #[instrument(skip(self))]
    pub async fn list(&self) -> Result<Vec<Backup>> {
        let cmd = serde_json::json!({
            "cmd": "list-backups"
        });
        self.client.command("backup", &cmd).await
    }

    /// Create a new backup.
    #[instrument(skip(self))]
    pub async fn create(&self) -> Result<Backup> {
        let cmd = serde_json::json!({
            "cmd": "backup"
        });
        let backups: Vec<Backup> = self.client.command("backup", &cmd).await?;
        backups
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No backup returned".to_string()))
    }

    /// Delete a backup by filename.
    #[instrument(skip(self))]
    pub async fn delete(&self, filename: &str) -> Result<()> {
        let cmd = serde_json::json!({
            "cmd": "delete-backup",
            "filename": filename
        });
        let _: serde_json::Value = self.client.command("backup", &cmd).await?;
        Ok(())
    }

    /// Restore from a backup.
    /// WARNING: This will restart the controller and apply the backup configuration.
    #[instrument(skip(self))]
    pub async fn restore(&self, filename: &str) -> Result<()> {
        let cmd = serde_json::json!({
            "cmd": "restore",
            "filename": filename
        });
        let _: serde_json::Value = self.client.command("backup", &cmd).await?;
        Ok(())
    }

    /// Download backup file contents.
    #[instrument(skip(self))]
    pub async fn download(&self, filename: &str) -> Result<String> {
        self.client.get_raw(&format!("dl/backup/{filename}")).await
    }

    /// Get the latest backup.
    #[instrument(skip(self))]
    pub async fn get_latest(&self) -> Result<Option<Backup>> {
        let backups = self.list().await?;
        Ok(backups.into_iter().max_by_key(|b| b.timestamp))
    }

    /// Get backup settings.
    #[instrument(skip(self))]
    pub async fn get_settings(&self) -> Result<BackupSettings> {
        let settings: Vec<BackupSettings> = self.client.get("rest/setting/backup").await?;
        settings
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No settings returned".to_string()))
    }

    /// Update backup settings.
    #[instrument(skip(self, settings))]
    pub async fn update_settings(&self, settings: &BackupSettings) -> Result<BackupSettings> {
        let result: Vec<BackupSettings> = self.client.put("rest/setting/backup", settings).await?;
        result
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No settings returned".to_string()))
    }

    /// Enable automatic backups.
    #[instrument(skip(self))]
    pub async fn enable_auto_backup(
        &self,
        retention_days: u32,
        max_backups: u32,
    ) -> Result<BackupSettings> {
        let settings = BackupSettings {
            enabled: true,
            cron_expr: None,
            retention_days,
            max_backups,
            cloud_backup_enabled: false,
        };
        self.update_settings(&settings).await
    }

    /// Disable automatic backups.
    #[instrument(skip(self))]
    pub async fn disable_auto_backup(&self) -> Result<BackupSettings> {
        let mut settings = self.get_settings().await?;
        settings.enabled = false;
        self.update_settings(&settings).await
    }

    /// Export site configuration as JSON.
    #[instrument(skip(self))]
    pub async fn export_site_config(&self) -> Result<serde_json::Value> {
        self.client.get("rest/setting").await
    }

    /// Get system configuration.
    #[instrument(skip(self))]
    pub async fn get_system_config(&self) -> Result<serde_json::Value> {
        self.client.get("stat/sysinfo").await
    }

    /// Prune old backups (keep only the most recent N).
    #[instrument(skip(self))]
    pub async fn prune(&self, keep_count: usize) -> Result<Vec<String>> {
        let mut backups = self.list().await?;
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let mut deleted = Vec::new();
        for backup in backups.into_iter().skip(keep_count) {
            self.delete(&backup.filename).await?;
            deleted.push(backup.filename);
        }

        Ok(deleted)
    }
}
