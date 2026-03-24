//! Backup and restore models.
//!
//! # Example
//!
//! ```rust
//! use threatflux_unifi_sdk::models::backup::{Backup, BackupSettings};
//!
//! // Create backup settings
//! let settings = BackupSettings {
//!     enabled: true,
//!     retention_days: 30,
//!     max_backups: 10,
//!     ..Default::default()
//! };
//!
//! assert!(settings.enabled);
//! assert_eq!(settings.retention_days, 30);
//! ```

use serde::{Deserialize, Serialize};

/// Backup metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    /// Backup filename.
    #[serde(rename = "filename")]
    pub filename: String,

    /// Backup size in bytes.
    #[serde(default)]
    pub size: u64,

    /// Creation timestamp.
    #[serde(rename = "datetime", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// Unix timestamp.
    #[serde(rename = "time", default)]
    pub timestamp: u64,

    /// Controller version at backup time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Backup format version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// Days retained.
    #[serde(default)]
    pub days: u32,
}

/// Backup settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSettings {
    /// Auto-backup enabled.
    #[serde(default)]
    pub enabled: bool,

    /// Backup schedule (cron-like).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron_expr: Option<String>,

    /// Days to retain backups.
    #[serde(default = "default_retention")]
    pub retention_days: u32,

    /// Maximum number of backups to keep.
    #[serde(default = "default_max_backups")]
    pub max_backups: u32,

    /// Cloud backup enabled.
    #[serde(default)]
    pub cloud_backup_enabled: bool,
}

const fn default_retention() -> u32 {
    30
}

const fn default_max_backups() -> u32 {
    10
}

impl Default for BackupSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            cron_expr: None,
            retention_days: 30,
            max_backups: 10,
            cloud_backup_enabled: false,
        }
    }
}

/// Site configuration export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    /// Configuration data (JSON).
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_settings_default() {
        let settings = BackupSettings::default();
        assert!(settings.enabled);
        assert_eq!(settings.retention_days, 30);
        assert_eq!(settings.max_backups, 10);
        assert!(!settings.cloud_backup_enabled);
        assert!(settings.cron_expr.is_none());
    }

    #[test]
    fn test_backup_serialization() {
        let backup = Backup {
            filename: "autobackup_2024-01-15.unf".to_string(),
            size: 1024000,
            created_at: Some("2024-01-15 12:00:00".to_string()),
            timestamp: 1705320000,
            version: Some("7.5.187".to_string()),
            format: Some("6".to_string()),
            days: 30,
        };

        let json = serde_json::to_string(&backup).unwrap();
        assert!(json.contains("autobackup_2024-01-15.unf"));
        assert!(json.contains("1024000"));

        let deserialized: Backup = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.filename, backup.filename);
        assert_eq!(deserialized.size, backup.size);
        assert_eq!(deserialized.timestamp, backup.timestamp);
    }

    #[test]
    fn test_backup_settings_serialization() {
        let settings = BackupSettings {
            enabled: true,
            cron_expr: Some("0 0 * * *".to_string()),
            retention_days: 14,
            max_backups: 5,
            cloud_backup_enabled: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"retention_days\":14"));

        let deserialized: BackupSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.retention_days, 14);
        assert_eq!(deserialized.max_backups, 5);
        assert!(deserialized.cloud_backup_enabled);
    }

    #[test]
    fn test_site_config() {
        let config = SiteConfig {
            data: serde_json::json!({
                "site_name": "default",
                "setting": "value"
            }),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("site_name"));
    }
}
