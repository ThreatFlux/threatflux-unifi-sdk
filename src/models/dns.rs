//! DNS record and filtering models.
//!
//! # Example
//!
//! ```rust
//! use threatflux_unifi_sdk::models::dns::DnsRecord;
//!
//! // Create an A record
//! let record = DnsRecord::a_record("server.local", "192.168.1.10")
//!     .with_ttl(3600);
//!
//! assert_eq!(record.key, "server.local");
//! assert_eq!(record.value, "192.168.1.10");
//! assert_eq!(record.ttl, Some(3600));
//!
//! // Create a CNAME record
//! let cname = DnsRecord::cname_record("www.local", "server.local");
//! assert_eq!(cname.key, "www.local");
//! ```

use serde::{Deserialize, Serialize};

use crate::types::DnsRecordType;

/// Local DNS record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    /// Unique identifier.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Record key/hostname.
    pub key: String,

    /// Record value (IP address or target).
    pub value: String,

    /// Record type (A, CNAME, etc.).
    #[serde(rename = "record_type", default)]
    pub record_type: DnsRecordType,

    /// Whether this record is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// TTL in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,

    /// Site ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
}

const fn default_true() -> bool {
    true
}

impl DnsRecord {
    /// Create a new A record.
    #[must_use]
    pub fn a_record(hostname: impl Into<String>, ip: impl Into<String>) -> Self {
        Self {
            id: None,
            key: hostname.into(),
            value: ip.into(),
            record_type: DnsRecordType::A,
            enabled: true,
            ttl: None,
            site_id: None,
        }
    }

    /// Create a new CNAME record.
    #[must_use]
    pub fn cname_record(alias: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            id: None,
            key: alias.into(),
            value: target.into(),
            record_type: DnsRecordType::Cname,
            enabled: true,
            ttl: None,
            site_id: None,
        }
    }

    /// Set TTL.
    #[must_use]
    pub const fn with_ttl(mut self, ttl: u32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Enable or disable the record.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_record() {
        let record = DnsRecord::a_record("server.local", "192.168.1.10");
        assert_eq!(record.key, "server.local");
        assert_eq!(record.value, "192.168.1.10");
        assert_eq!(record.record_type, DnsRecordType::A);
        assert!(record.enabled);
        assert!(record.id.is_none());
    }

    #[test]
    fn test_cname_record() {
        let record = DnsRecord::cname_record("www.local", "server.local");
        assert_eq!(record.record_type, DnsRecordType::Cname);
        assert_eq!(record.key, "www.local");
        assert_eq!(record.value, "server.local");
    }

    #[test]
    fn test_record_with_ttl() {
        let record = DnsRecord::a_record("test.local", "192.168.1.1").with_ttl(3600);
        assert_eq!(record.ttl, Some(3600));
    }

    #[test]
    fn test_record_with_enabled() {
        let record = DnsRecord::a_record("test.local", "192.168.1.1").with_enabled(false);
        assert!(!record.enabled);
    }

    #[test]
    fn test_dns_record_serialization() {
        let record = DnsRecord::a_record("host.local", "10.0.0.1");
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("host.local"));
        assert!(json.contains("10.0.0.1"));

        let deserialized: DnsRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.key, "host.local");
        assert_eq!(deserialized.value, "10.0.0.1");
    }
}
