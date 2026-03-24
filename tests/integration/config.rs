use std::env;

use anyhow::{Result, anyhow};
use threatflux_unifi_sdk::{UnifiClient, UnifiConfig};
use uuid::Uuid;

const REQUIRED_ENV: [&str; 3] = ["UNIFI_HOST", "UNIFI_USERNAME", "UNIFI_PASSWORD"];

#[derive(Debug, Clone)]
pub struct TestConfig {
    host: String,
    username: String,
    password: String,
    site: String,
    verify_ssl: bool,
    timeout_secs: u64,
}

impl TestConfig {
    pub fn from_env() -> Option<Self> {
        let host = env_var("UNIFI_HOST")?;
        let username = env_var("UNIFI_USERNAME")?;
        let password = env_var("UNIFI_PASSWORD")?;
        let site = env_var("UNIFI_SITE").unwrap_or_else(|| "default".to_string());
        let verify_ssl = env_bool("UNIFI_VERIFY_SSL").unwrap_or(false);
        let timeout_secs = env_u64("UNIFI_TIMEOUT_SECS").unwrap_or(30);

        Some(Self { host, username, password, site, verify_ssl, timeout_secs })
    }

    pub fn site(&self) -> &str {
        &self.site
    }

    pub async fn connect(&self) -> Result<UnifiClient> {
        let config = UnifiConfig::new(&self.host, &self.username, &self.password)
            .with_site(&self.site)
            .with_verify_ssl(self.verify_ssl)
            .with_timeout(self.timeout_secs);

        UnifiClient::connect(config).await.map_err(|err| anyhow!(err))
    }
}

pub fn require_config() -> Option<TestConfig> {
    let config = TestConfig::from_env();
    if config.is_none() {
        let missing =
            REQUIRED_ENV.iter().filter(|key| env_var(key).is_none()).copied().collect::<Vec<_>>();
        eprintln!("Skipping UniFi integration tests (missing env vars: {}).", missing.join(", "));
    }
    config
}

pub fn unique_name(prefix: &str) -> String {
    format!("{}_{}", prefix, Uuid::new_v4().simple())
}

pub fn unique_vlan_config() -> (u16, String, String, String) {
    let seed = Uuid::new_v4().as_u128();
    let vlan_id = 2000 + (seed % 800) as u16;
    let octet = 10 + (seed % 200) as u8;
    let subnet = format!("10.250.{octet}.0/24");
    let dhcp_start = format!("10.250.{octet}.10");
    let dhcp_stop = format!("10.250.{octet}.200");
    (vlan_id, subnet, dhcp_start, dhcp_stop)
}

fn env_var(name: &str) -> Option<String> {
    env::var(name).ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn env_bool(name: &str) -> Option<bool> {
    let value = env_var(name)?;
    let normalized = value.to_ascii_lowercase();
    match normalized.as_str() {
        "true" | "1" | "yes" | "y" | "on" => Some(true),
        "false" | "0" | "no" | "n" | "off" => Some(false),
        _ => None,
    }
}

fn env_u64(name: &str) -> Option<u64> {
    env_var(name).and_then(|value| value.parse::<u64>().ok())
}
