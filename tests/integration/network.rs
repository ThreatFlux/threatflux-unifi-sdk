use anyhow::{Result, anyhow};
use threatflux_unifi_sdk::NetworkService;

use super::config::{require_config, unique_name, unique_vlan_config};

#[tokio::test]
#[ignore = "requires UniFi controller"]
async fn test_network_lifecycle() -> Result<()> {
    let Some(config) = require_config() else {
        return Ok(());
    };

    let client = config.connect().await?;
    let network_service = NetworkService::new(&client);

    let name = unique_name("TFX_TEST_VLAN");
    let (vlan_id, subnet, dhcp_start, dhcp_stop) = unique_vlan_config();

    let created = network_service
        .create_vlan(&name, vlan_id, &subnet, Some(&dhcp_start), Some(&dhcp_stop))
        .await?;
    let network_id = created.id.clone().ok_or_else(|| anyhow!("created network missing id"))?;

    let result = async {
        let fetched = network_service.get(&network_id).await?;
        assert_eq!(fetched.name, name);

        let by_name = network_service.get_by_name(&name).await?;
        assert!(by_name.is_some(), "network lookup by name failed");

        Ok(())
    }
    .await;

    let _ = network_service.delete(&network_id).await;
    result
}
