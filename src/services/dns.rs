//! DNS management service.

use tracing::instrument;

use crate::client::UnifiClient;
use crate::error::{Result, UnifiError};
use crate::models::dns::DnsRecord;
use crate::types::DnsFilterLevel;

/// Service for managing local DNS records.
pub struct DnsService<'a> {
    client: &'a UnifiClient,
}

impl<'a> DnsService<'a> {
    /// Create a new DNS service.
    #[must_use]
    pub const fn new(client: &'a UnifiClient) -> Self {
        Self { client }
    }

    /// List all DNS records.
    #[instrument(skip(self))]
    pub async fn list(&self) -> Result<Vec<DnsRecord>> {
        self.client.get("rest/dnsrecord").await
    }

    /// Get a DNS record by ID.
    #[instrument(skip(self))]
    pub async fn get(&self, id: &str) -> Result<DnsRecord> {
        let records: Vec<DnsRecord> = self.client.get(&format!("rest/dnsrecord/{id}")).await?;
        records
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::NotFound(format!("DNS record {id} not found")))
    }

    /// Get DNS record by hostname.
    #[instrument(skip(self))]
    pub async fn get_by_hostname(&self, hostname: &str) -> Result<Option<DnsRecord>> {
        let records = self.list().await?;
        Ok(records.into_iter().find(|r| r.key == hostname))
    }

    /// Create a DNS record.
    #[instrument(skip(self, record))]
    pub async fn create(&self, record: &DnsRecord) -> Result<DnsRecord> {
        let records: Vec<DnsRecord> = self.client.post("rest/dnsrecord", record).await?;
        records
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No record returned".to_string()))
    }

    /// Update a DNS record.
    #[instrument(skip(self, record))]
    pub async fn update(&self, id: &str, record: &DnsRecord) -> Result<DnsRecord> {
        let records: Vec<DnsRecord> =
            self.client.put(&format!("rest/dnsrecord/{id}"), record).await?;
        records
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No record returned".to_string()))
    }

    /// Delete a DNS record.
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("rest/dnsrecord/{id}")).await
    }

    /// Create an A record.
    #[instrument(skip(self))]
    pub async fn create_a_record(&self, hostname: &str, ip: &str) -> Result<DnsRecord> {
        let record = DnsRecord::a_record(hostname, ip);
        self.create(&record).await
    }

    /// Create a CNAME record.
    #[instrument(skip(self))]
    pub async fn create_cname_record(&self, alias: &str, target: &str) -> Result<DnsRecord> {
        let record = DnsRecord::cname_record(alias, target);
        self.create(&record).await
    }

    /// Enable or disable a DNS record.
    #[instrument(skip(self))]
    pub async fn set_enabled(&self, id: &str, enabled: bool) -> Result<DnsRecord> {
        let mut record = self.get(id).await?;
        record.enabled = enabled;
        self.update(id, &record).await
    }

    /// Update the IP address for an A record.
    #[instrument(skip(self))]
    pub async fn update_ip(&self, id: &str, new_ip: &str) -> Result<DnsRecord> {
        let mut record = self.get(id).await?;
        record.value = new_ip.to_string();
        self.update(id, &record).await
    }

    /// Get DNS settings (raw).
    #[instrument(skip(self))]
    pub async fn get_settings(&self) -> Result<serde_json::Value> {
        let settings: Vec<serde_json::Value> = self.client.get("rest/setting/dns").await?;
        settings
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No DNS settings returned".to_string()))
    }

    /// Update DNS settings (raw).
    #[instrument(skip(self, settings))]
    pub async fn update_settings(&self, settings: &serde_json::Value) -> Result<serde_json::Value> {
        let id = settings
            .get("_id")
            .and_then(|value| value.as_str())
            .ok_or_else(|| UnifiError::InvalidResponse("DNS settings missing _id".to_string()))?;
        let updated: Vec<serde_json::Value> =
            self.client.put(&format!("rest/setting/dns/{id}"), settings).await?;
        updated
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No DNS settings returned".to_string()))
    }

    /// Set upstream DNS servers.
    #[instrument(skip(self, servers))]
    pub async fn set_upstream_dns(&self, servers: &[String]) -> Result<serde_json::Value> {
        let mut settings = self.get_settings().await?;
        let mut ordered = servers.iter();

        for key in ["dns1", "dns2", "dns3", "dns4"] {
            if let Some(server) = ordered.next() {
                settings[key] = serde_json::Value::String(server.clone());
            } else {
                settings.as_object_mut().map(|obj| obj.remove(key));
            }
        }

        settings["upstream_dns"] = serde_json::json!(servers);
        self.update_settings(&settings).await
    }

    /// Enable or disable DNS filtering.
    #[instrument(skip(self))]
    pub async fn set_dns_filtering(
        &self,
        enabled: bool,
        level: DnsFilterLevel,
    ) -> Result<serde_json::Value> {
        let mut settings = self.get_settings().await?;
        settings["filtering_enabled"] = serde_json::Value::Bool(enabled);
        settings["filtering_level"] = serde_json::Value::String(level.to_string());
        self.update_settings(&settings).await
    }
}
