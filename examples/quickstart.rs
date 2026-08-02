use std::env;

use threatflux_unifi_sdk::{NetworkService, UnifiClient, UnifiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = UnifiConfig::new(
        env::var("UNIFI_HOST")?,
        env::var("UNIFI_USERNAME")?,
        env::var("UNIFI_PASSWORD")?,
    )
    .with_site(env::var("UNIFI_SITE").unwrap_or_else(|_| "default".to_owned()))
    .with_verify_ssl(true)
    .with_timeout(30);

    let client = UnifiClient::connect(config).await?;
    let networks = NetworkService::new(&client).list().await?;

    for network in networks {
        println!("{}", network.name);
    }

    Ok(())
}
