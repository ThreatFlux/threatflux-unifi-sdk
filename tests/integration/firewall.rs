use anyhow::{Result, anyhow};
use threatflux_unifi_sdk::FirewallService;

use super::config::{require_config, unique_name};

#[tokio::test]
#[ignore = "requires UniFi controller"]
async fn test_firewall_group_lifecycle() -> Result<()> {
    let Some(config) = require_config() else {
        return Ok(());
    };

    let client = config.connect().await?;
    let firewall_service = FirewallService::new(&client);

    let name = unique_name("TFX_TEST_GROUP");
    let group = firewall_service
        .create_address_group(
            &name,
            vec![
                "10.0.0.0/8".to_string(),
                "172.16.0.0/12".to_string(),
                "192.168.0.0/16".to_string(),
            ],
        )
        .await?;

    let group_id = group.id.clone().ok_or_else(|| anyhow!("created firewall group missing id"))?;

    let result = async {
        let fetched = firewall_service.get_group(&group_id).await?;
        assert_eq!(fetched.name, name);
        Ok(())
    }
    .await;

    let _ = firewall_service.delete_group(&group_id).await;
    result
}
