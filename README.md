# ThreatFlux UniFi SDK

Native Rust SDK for UDM Pro and UniFi OS device automation.

## Features

- **Network/VLAN Management**: Create, update, and delete networks and VLANs
- **Firewall Management**: Manage firewall rules and groups across all rulesets
- **Port Forwarding**: Configure NAT and port forwarding rules
- **Traffic Management**: Traffic rules, app blocking, and bandwidth limiting
- **VPN Configuration**: WireGuard and site-to-site VPN setup
- **Client Management**: Block/unblock clients, assign fixed IPs
- **Device Management**: Adopt, configure, and monitor UniFi devices
- **DHCP & DNS**: Manage reservations and local DNS records
- **Infrastructure as Code**: Declarative YAML config with sync/diff/export
- **CLI**: `unifi-cli` commands for day-to-day automation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
threatflux-unifi-sdk = "0.4"
```

## Quick Start

```rust
use threatflux_unifi_sdk::{UnifiClient, UnifiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to UDM Pro
    let config = UnifiConfig::new("192.168.1.1", "admin", "password")
        .with_site("default")
        .with_verify_ssl(false);  // For self-signed certs

    let client = UnifiClient::connect(config).await?;

    // List all networks
    let networks: Vec<serde_json::Value> = client.get("rest/networkconf").await?;
    println!("Found {} networks", networks.len());

    // List active clients
    let clients: Vec<serde_json::Value> = client.get("stat/sta").await?;
    println!("Found {} active clients", clients.len());

    Ok(())
}
```

## Configuration

### Environment Variables

```bash
UNIFI_HOST=192.168.1.1
UNIFI_USERNAME=admin
UNIFI_PASSWORD=your-password
UNIFI_SITE=default
UNIFI_VERIFY_SSL=false
```

### Configuration Options

| Option         | Default   | Description                     |
| -------------- | --------- | ------------------------------- |
| `host`         | Required  | UniFi controller IP or hostname |
| `username`     | Required  | Admin username                  |
| `password`     | Required  | Admin password                  |
| `site`         | `default` | Site name                       |
| `verify_ssl`   | `false`   | Verify TLS certificates         |
| `timeout_secs` | `30`      | Request timeout in seconds      |

## Declarative Sync

Use YAML to define networks, firewall rules, port forwards, VPN settings, and more. Then run a sync to reconcile the
controller with the desired state.

```bash
cargo run --bin unifi-cli -- diff --config config/unifi.example.yaml
cargo run --bin unifi-cli -- sync --config config/unifi.example.yaml
```

The example config lives at `config/unifi.example.yaml`.

## CLI

The SDK ships a CLI binary named `threatflux` that exposes UniFi subcommands:

```bash
unifi-cli status
unifi-cli clients list
unifi-cli devices upgrade-all
```

## Controller Compatibility

| Controller Type | Supported | Notes          |
| --------------- | --------- | -------------- |
| UDM Pro         | ✅        | Primary target |
| UDM SE          | ✅        | Full support   |
| UDM             | ✅        | Full support   |
| Dream Router    | ✅        | Full support   |
| Cloud Key Gen2  | ✅        | Classic API    |
| Self-hosted     | ✅        | Classic API    |

## Feature Flags

- `full` (default): Enable all features
- `firewall`: Firewall management only
- `vpn`: VPN configuration only
- `traffic`: Traffic rules only
- `clients`: Client management only
- `devices`: Device management only
- `dhcp`: DHCP management only
- `dns`: DNS management only

## Services Reference

### NetworkService - Network/VLAN Management

```rust
use threatflux_unifi_sdk::{UnifiClient, NetworkService, Network};

let client = UnifiClient::connect(config).await?;
let networks = NetworkService::new(&client);

// List all networks
let all = networks.list().await?;

// Create a new VLAN
let vlan = Network::new_vlan("IoT Network", 100, "192.168.100.1/24");
let created = networks.create(&vlan).await?;

// Get corporate networks only
let corporate = networks.get_corporate_networks().await?;

// Delete a network
networks.delete("network_id").await?;
```

### FirewallService - Firewall Rules & Groups

```rust
use threatflux_unifi_sdk::{FirewallService, FirewallRule, FirewallAction};

let firewall = FirewallService::new(&client);

// List all firewall rules
let rules = firewall.list_rules().await?;

// Create a block rule
let rule = FirewallRule::new("Block IoT to LAN", FirewallAction::Drop)
    .with_src_network("iot_network_id")
    .with_dst_network("lan_network_id");
firewall.create_rule(&rule).await?;

// Enable/disable a rule
firewall.set_rule_enabled("rule_id", false).await?;

// Create firewall groups
let group = firewall.create_address_group("Blocked IPs", vec!["1.2.3.4", "5.6.7.8"]).await?;
```

### PortForwardService - NAT/Port Forwarding

```rust
use threatflux_unifi_sdk::{PortForwardService, PortForward};

let pf = PortForwardService::new(&client);

// List all port forwards
let rules = pf.list().await?;

// Create a port forward for a web server
let forward = PortForward::tcp("Web Server", 80, "192.168.1.100", 80);
pf.create(&forward).await?;

// Create with source restriction
let secure = PortForward::tcp("SSH", 22, "192.168.1.100", 22)
    .with_source("10.0.0.0/8");
pf.create(&secure).await?;
```

### ClientService - Client Management

```rust
use threatflux_unifi_sdk::ClientService;

let clients = ClientService::new(&client);

// List all clients
let all = clients.list().await?;

// Get online clients only
let online = clients.get_online().await?;

// Block a client
clients.block("aa:bb:cc:dd:ee:ff").await?;

// Assign a fixed IP
clients.set_fixed_ip("aa:bb:cc:dd:ee:ff", "192.168.1.50").await?;

// Kick a client (force reconnect)
clients.kick("aa:bb:cc:dd:ee:ff").await?;

// Authorize a guest
clients.authorize_guest("aa:bb:cc:dd:ee:ff", 60).await?; // 60 minutes
```

### DeviceService - Device Management

```rust
use threatflux_unifi_sdk::DeviceService;

let devices = DeviceService::new(&client);

// List all devices
let all = devices.list().await?;

// Get gateways only
let gateways = devices.get_gateways().await?;

// Adopt a device
devices.adopt("aa:bb:cc:dd:ee:ff").await?;

// Restart a device
devices.restart("aa:bb:cc:dd:ee:ff").await?;

// Upgrade firmware
devices.upgrade("aa:bb:cc:dd:ee:ff").await?;

// Enable locator LED
devices.locate("aa:bb:cc:dd:ee:ff", true).await?;
```

### VpnService - WireGuard & Site-to-Site VPN

```rust
use threatflux_unifi_sdk::{VpnService, WireGuardServer, WireGuardPeer, SiteVpn};

let vpn = VpnService::new(&client);

// Create a WireGuard server
let server = WireGuardServer::new("Home VPN", "10.10.0.1/24")
    .with_port(51820)
    .with_dns(vec!["1.1.1.1".to_string()]);
vpn.create_wireguard_server(&server).await?;

// Add a WireGuard peer
let peer = WireGuardPeer::new("Phone", "public_key_here")
    .with_allowed_ips(vec!["10.10.0.2/32".to_string()]);
vpn.create_wireguard_peer("server_id", &peer).await?;

// Create site-to-site VPN
let s2s = SiteVpn::new("Branch Office", "vpn.branch.example.com")
    .with_remote_subnets(vec!["192.168.100.0/24".to_string()])
    .with_psk("pre_shared_key");
vpn.create_site_vpn(&s2s).await?;
```

### TrafficService - Traffic Rules & Bandwidth

```rust
use threatflux_unifi_sdk::{TrafficService, TrafficRule};

let traffic = TrafficService::new(&client);

// Block social media for all clients
let block = TrafficRule::block("Block Social Media")
    .for_all_clients()
    .blocking_categories(vec!["social".to_string()]);
traffic.create(&block).await?;

// Rate limit a client
let limit = TrafficRule::rate_limit("Limit Guest", 10000, 5000)  // 10Mbps down, 5Mbps up
    .for_client("aa:bb:cc:dd:ee:ff");
traffic.create(&limit).await?;

// Block specific domains
let domains = TrafficRule::block("Block Sites")
    .blocking_domains(vec!["example.com".to_string()]);
traffic.create(&domains).await?;
```

### DhcpService - DHCP Reservations

```rust
use threatflux_unifi_sdk::{DhcpService, DhcpReservation};

let dhcp = DhcpService::new(&client);

// List reservations
let reservations = dhcp.list_reservations().await?;

// Create a reservation
let res = DhcpReservation::new("aa:bb:cc:dd:ee:ff", "192.168.1.100")
    .with_name("Server");
dhcp.create_reservation(&res).await?;

// Get active leases
let leases = dhcp.list_leases().await?;
```

### DnsService - Local DNS Records

```rust
use threatflux_unifi_sdk::{DnsService, DnsRecord};

let dns = DnsService::new(&client);

// Create an A record
let record = DnsRecord::a_record("server.local", "192.168.1.100")
    .with_ttl(3600);
dns.create(&record).await?;

// Create a CNAME
let cname = DnsRecord::cname_record("www.local", "server.local");
dns.create(&cname).await?;
```

### RoutingService - Static Routes

```rust
use threatflux_unifi_sdk::{RoutingService, StaticRoute};

let routing = RoutingService::new(&client);

// Create a static route
let route = StaticRoute::via_gateway("10.0.0.0/8", "192.168.1.254")
    .with_name("VPN Route");
routing.create_static_route(&route).await?;

// Create a blackhole route
let blackhole = StaticRoute::blackhole("192.168.100.0/24");
routing.create_static_route(&blackhole).await?;

// Get route table from gateway
let table = routing.get_route_table("gateway_mac").await?;
```

### BackupService - Configuration Backup

```rust
use threatflux_unifi_sdk::BackupService;

let backup = BackupService::new(&client);

// List backups
let backups = backup.list().await?;

// Create a new backup
let new_backup = backup.create().await?;

// Get latest backup
let latest = backup.get_latest().await?;

// Enable auto-backup
backup.enable_auto_backup(30, 10).await?;  // 30 days retention, max 10

// Prune old backups (keep 5)
let deleted = backup.prune(5).await?;
```

### SiteService - Site Management

```rust
use threatflux_unifi_sdk::SiteService;

let sites = SiteService::new(&client);

// List all sites
let all = sites.list().await?;

// Get current site
let current = sites.get_current().await?;

// Get site statistics
let stats = sites.get_stats().await?;

// Get system info
let info = sites.get_system_info().await?;

// Get recent events
let events = sites.get_events(100).await?;

// Get active alarms
let alarms = sites.get_alarms().await?;
```

## Low-Level API

### Making Raw Requests

```rust
// GET request
let data: Vec<Network> = client.get("rest/networkconf").await?;

// POST request
let new_network = client.post("rest/networkconf", &network_config).await?;

// PUT request
let updated = client.put(&format!("rest/networkconf/{}", id), &config).await?;

// DELETE request
client.delete(&format!("rest/networkconf/{}", id)).await?;

// Command request (POST to /cmd/{manager})
let result = client.command("stamgr", &block_command).await?;
```

### Authentication

The SDK handles authentication automatically:

1. Detects controller type (UniFi OS vs Classic)
2. Performs login with username/password
3. Extracts and manages CSRF tokens
4. Handles session expiration with automatic re-login

## Error Handling

```rust
use threatflux_unifi_sdk::{UnifiError, Result};

match client.get::<Vec<Network>>("rest/networkconf").await {
    Ok(networks) => println!("Found {} networks", networks.len()),
    Err(UnifiError::AuthenticationFailed(msg)) => eprintln!("Auth failed: {}", msg),
    Err(UnifiError::SessionExpired) => eprintln!("Session expired"),
    Err(UnifiError::NotFound(resource)) => eprintln!("Not found: {}", resource),
    Err(UnifiError::RateLimited { retry_after_secs }) => {
        eprintln!("Rate limited, retry after {:?}s", retry_after_secs);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Crate Layout

```text
threatflux-unifi-sdk/
├── src/
│   ├── bin/        # CLI binary entry point
│   ├── config/     # Configuration types and YAML parsing
│   ├── models/     # UniFi API data models
│   ├── services/   # Service modules (network, firewall, VPN, etc.)
│   ├── sync/       # Declarative sync/diff engine
│   ├── client.rs   # UnifiClient HTTP wrapper
│   ├── error.rs    # Error types
│   ├── types.rs    # Shared type aliases
│   └── lib.rs
└── Cargo.toml
```

## Security Notes

- **Self-signed certificates**: Most UniFi controllers use self-signed certs. Set `verify_ssl: false` to accept them.
- **Credentials**: Never hardcode credentials. Use environment variables or secure vaults.
- **Network access**: The SDK requires network access to the controller's management interface.

## License

MIT
