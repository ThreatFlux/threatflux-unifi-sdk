use anyhow::Result;
use threatflux_unifi_sdk::SiteService;

use super::config::require_config;

#[tokio::test]
#[ignore = "requires UniFi controller"]
async fn test_unifi_smoke() -> Result<()> {
    let Some(config) = require_config() else {
        return Ok(());
    };

    let client = config.connect().await?;
    let site_service = SiteService::new(&client);

    let info = site_service.get_system_info().await?;
    assert!(
        info.version.is_some() || info.name.is_some() || info.hostname.is_some(),
        "system info should include at least one identifier"
    );

    let sites = site_service.list().await?;
    assert!(!sites.is_empty(), "expected at least one site");

    let current = site_service.get_current().await?;
    assert_eq!(current.name, config.site());

    assert!(client.controller_type().await.is_some());

    Ok(())
}
