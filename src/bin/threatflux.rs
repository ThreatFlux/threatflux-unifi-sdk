use clap::{Args, Parser, Subcommand, ValueEnum};

use threatflux_unifi_sdk::config::load_config;
use threatflux_unifi_sdk::sync::{
    SyncOptions, apply as sync_apply, diff as sync_diff, export_config,
};
use threatflux_unifi_sdk::types::{FirewallAction, FirewallRuleset, Protocol, Timeframe};
use threatflux_unifi_sdk::{
    ClientService, FirewallService, PortForward, PortForwardService, SiteService, TrafficService,
    UnifiClient, UnifiConfig, VpnService,
};

#[derive(Debug, Parser)]
#[command(name = "unifi-cli")]
#[command(about = "UniFi automation CLI for UDM Pro and UniFi OS devices", version)]
struct Cli {
    #[command(flatten)]
    connection: ConnectionArgs,

    #[command(subcommand)]
    command: UnifiCommand,
}

#[derive(Debug, Args, Clone)]
struct ConnectionArgs {
    /// `UniFi` controller host (IP or hostname)
    #[arg(long, env = "UNIFI_HOST")]
    host: Option<String>,

    /// `UniFi` username
    #[arg(long, env = "UNIFI_USERNAME")]
    username: Option<String>,

    /// `UniFi` password
    #[arg(long, env = "UNIFI_PASSWORD")]
    password: Option<String>,

    /// `UniFi` site (default: default)
    #[arg(long, env = "UNIFI_SITE", default_value = "default")]
    site: String,

    /// Verify SSL certificates
    #[arg(long, env = "UNIFI_VERIFY_SSL", default_value_t = false)]
    verify_ssl: bool,

    /// Request timeout seconds
    #[arg(long, env = "UNIFI_TIMEOUT", default_value_t = 30)]
    timeout_secs: u64,
}

impl ConnectionArgs {
    fn to_config(&self) -> anyhow::Result<UnifiConfig> {
        let host =
            self.host.clone().ok_or_else(|| anyhow::anyhow!("UNIFI_HOST or --host is required"))?;
        let username = self
            .username
            .clone()
            .ok_or_else(|| anyhow::anyhow!("UNIFI_USERNAME or --username is required"))?;
        let password = self
            .password
            .clone()
            .ok_or_else(|| anyhow::anyhow!("UNIFI_PASSWORD or --password is required"))?;

        Ok(UnifiConfig::new(host, username, password)
            .with_site(self.site.clone())
            .with_verify_ssl(self.verify_ssl)
            .with_timeout(self.timeout_secs))
    }
}

#[derive(Debug, Subcommand)]
enum UnifiCommand {
    /// Sync `UniFi` configuration from YAML
    Sync(SyncArgs),

    /// Show diff between desired config and controller
    Diff(SyncArgs),

    /// Export current controller configuration
    Export(ExportArgs),

    /// Show controller status and health
    Status,

    /// Client management
    Clients(ClientArgs),

    /// Device management
    Devices(DeviceArgs),

    /// VPN management
    Vpn(VpnArgs),

    /// Firewall management
    Firewall(FirewallArgs),

    /// Port forwarding management
    PortForward(PortForwardArgs),

    /// Traffic statistics
    Traffic(TrafficArgs),
}

#[derive(Debug, Args)]
struct SyncArgs {
    /// Config file path
    #[arg(long)]
    config: String,

    /// Dry-run (no changes)
    #[arg(long)]
    dry_run: bool,

    /// Remove resources not in config
    #[arg(long)]
    prune: bool,
}

#[derive(Debug, Args)]
struct ExportArgs {
    /// Output format (yaml or json)
    #[arg(long, default_value = "yaml")]
    format: String,
}

#[derive(Debug, Args)]
struct ClientArgs {
    #[command(subcommand)]
    command: ClientCommand,
}

#[derive(Debug, Subcommand)]
enum ClientCommand {
    List,
    Active,
    Block { mac: String },
    Unblock { mac: String },
}

#[derive(Debug, Args)]
struct DeviceArgs {
    #[command(subcommand)]
    command: DeviceCommand,
}

#[derive(Debug, Subcommand)]
enum DeviceCommand {
    List,
    Restart { mac: String },
    UpgradeAll,
}

#[derive(Debug, Args)]
struct VpnArgs {
    #[command(subcommand)]
    command: VpnCommand,
}

#[derive(Debug, Subcommand)]
enum VpnCommand {
    Status,
    Wireguard(WireGuardArgs),
}

#[derive(Debug, Args)]
struct WireGuardArgs {
    #[command(subcommand)]
    command: WireGuardCommand,
}

#[derive(Debug, Subcommand)]
enum WireGuardCommand {
    ListClients,
    AddClient {
        name: String,
        #[arg(long)]
        public_key: String,
        #[arg(long)]
        server_id: Option<String>,
        #[arg(long, default_value = "split")]
        tunnel: TunnelMode,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TunnelMode {
    Split,
    Full,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FirewallRulesetArg {
    WanIn,
    WanOut,
    WanLocal,
    LanIn,
    LanOut,
    LanLocal,
    GuestIn,
    GuestOut,
    GuestLocal,
    InterVlan,
}

impl From<FirewallRulesetArg> for FirewallRuleset {
    fn from(value: FirewallRulesetArg) -> Self {
        match value {
            FirewallRulesetArg::WanIn => Self::WanIn,
            FirewallRulesetArg::WanOut => Self::WanOut,
            FirewallRulesetArg::WanLocal => Self::WanLocal,
            FirewallRulesetArg::LanIn => Self::LanIn,
            FirewallRulesetArg::LanOut => Self::LanOut,
            FirewallRulesetArg::LanLocal => Self::LanLocal,
            FirewallRulesetArg::GuestIn => Self::GuestIn,
            FirewallRulesetArg::GuestOut => Self::GuestOut,
            FirewallRulesetArg::GuestLocal => Self::GuestLocal,
            FirewallRulesetArg::InterVlan => Self::InterVlan,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FirewallActionArg {
    Accept,
    Drop,
    Reject,
}

impl From<FirewallActionArg> for FirewallAction {
    fn from(value: FirewallActionArg) -> Self {
        match value {
            FirewallActionArg::Accept => Self::Accept,
            FirewallActionArg::Drop => Self::Drop,
            FirewallActionArg::Reject => Self::Reject,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ProtocolArg {
    Tcp,
    Udp,
    TcpUdp,
}

impl From<ProtocolArg> for Protocol {
    fn from(value: ProtocolArg) -> Self {
        match value {
            ProtocolArg::Tcp => Self::Tcp,
            ProtocolArg::Udp => Self::Udp,
            ProtocolArg::TcpUdp => Self::TcpUdp,
        }
    }
}

#[derive(Debug, Args)]
struct FirewallArgs {
    #[command(subcommand)]
    command: FirewallCommand,
}

#[derive(Debug, Subcommand)]
enum FirewallCommand {
    ListRules {
        #[arg(long)]
        ruleset: Option<FirewallRulesetArg>,
    },
    AddRule {
        name: String,
        #[arg(long)]
        ruleset: FirewallRulesetArg,
        #[arg(long, default_value = "drop")]
        action: FirewallActionArg,
        #[arg(long)]
        src: Option<String>,
        #[arg(long)]
        dst: Option<String>,
    },
}

#[derive(Debug, Args)]
struct PortForwardArgs {
    #[command(subcommand)]
    command: PortForwardCommand,
}

#[derive(Debug, Subcommand)]
enum PortForwardCommand {
    List,
    Add {
        name: String,
        #[arg(long)]
        port: u16,
        #[arg(long)]
        fwd: String,
        #[arg(long = "fwd-port")]
        fwd_port: u16,
        #[arg(long, default_value = "tcp")]
        protocol: ProtocolArg,
    },
}

#[derive(Debug, Args)]
struct TrafficArgs {
    #[command(subcommand)]
    command: TrafficCommand,
}

#[derive(Debug, Subcommand)]
enum TrafficCommand {
    Stats { hours: u32 },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        UnifiCommand::Sync(args) => handle_sync(args).await,
        UnifiCommand::Diff(args) => handle_diff(args).await,
        UnifiCommand::Export(args) => handle_export(&cli.connection, args).await,
        UnifiCommand::Status => handle_status(&cli.connection).await,
        UnifiCommand::Clients(args) => handle_clients(&cli.connection, args).await,
        UnifiCommand::Devices(args) => handle_devices(&cli.connection, args).await,
        UnifiCommand::Vpn(args) => handle_vpn(&cli.connection, args).await,
        UnifiCommand::Firewall(args) => handle_firewall(&cli.connection, args).await,
        UnifiCommand::PortForward(args) => handle_port_forward(&cli.connection, args).await,
        UnifiCommand::Traffic(args) => handle_traffic(&cli.connection, args).await,
    }
}

async fn connect_from_args(connection: &ConnectionArgs) -> anyhow::Result<UnifiClient> {
    let client = UnifiClient::connect(connection.to_config()?).await?;
    Ok(client)
}

async fn handle_sync(args: SyncArgs) -> anyhow::Result<()> {
    let config = load_config(&args.config)?;
    let client = UnifiClient::connect(config.unifi.to_client_config()).await?;
    let report =
        sync_apply(&client, &config, SyncOptions { dry_run: args.dry_run, prune: args.prune })
            .await?;
    println!("{} changes (dry_run={})", report.changes.len(), args.dry_run);
    for change in report.changes {
        println!("{:?} {:?}: {}", change.action, change.resource, change.name);
    }
    Ok(())
}

async fn handle_diff(args: SyncArgs) -> anyhow::Result<()> {
    let config = load_config(&args.config)?;
    let client = UnifiClient::connect(config.unifi.to_client_config()).await?;
    let plan = sync_diff(&client, &config).await?;
    for change in plan.changes {
        println!("{:?} {:?}: {}", change.action, change.resource, change.name);
    }
    Ok(())
}

async fn handle_export(connection: &ConnectionArgs, args: ExportArgs) -> anyhow::Result<()> {
    let client = connect_from_args(connection).await?;
    let export = export_config(&client).await?;
    match args.format.as_str() {
        "json" => println!("{}", serde_json::to_string_pretty(&export)?),
        _ => println!("{}", serde_yaml_ng::to_string(&export)?),
    }
    Ok(())
}

async fn handle_status(connection: &ConnectionArgs) -> anyhow::Result<()> {
    let client = connect_from_args(connection).await?;
    let site = SiteService::new(&client).get_health().await?;
    let info = SiteService::new(&client).get_system_info().await?;
    println!("{}", serde_json::to_string_pretty(&(site, info))?);
    Ok(())
}

async fn handle_clients(connection: &ConnectionArgs, args: ClientArgs) -> anyhow::Result<()> {
    let client = connect_from_args(connection).await?;
    let service = ClientService::new(&client);
    match args.command {
        ClientCommand::List => {
            let clients = service.list_all().await?;
            println!("{}", serde_json::to_string_pretty(&clients)?);
        }
        ClientCommand::Active => {
            let clients = service.list_online().await?;
            println!("{}", serde_json::to_string_pretty(&clients)?);
        }
        ClientCommand::Block { mac } => {
            service.block(&mac).await?;
            println!("blocked {mac}");
        }
        ClientCommand::Unblock { mac } => {
            service.unblock(&mac).await?;
            println!("unblocked {mac}");
        }
    }
    Ok(())
}

async fn handle_devices(connection: &ConnectionArgs, args: DeviceArgs) -> anyhow::Result<()> {
    let client = connect_from_args(connection).await?;
    let service = threatflux_unifi_sdk::DeviceService::new(&client);
    match args.command {
        DeviceCommand::List => {
            let devices = service.list().await?;
            println!("{}", serde_json::to_string_pretty(&devices)?);
        }
        DeviceCommand::Restart { mac } => {
            service.restart(&mac).await?;
            println!("restarted {mac}");
        }
        DeviceCommand::UpgradeAll => {
            let devices = service.list_upgradable().await?;
            for device in devices {
                service.upgrade(&device.mac_address).await?;
                println!("upgrade requested for {}", device.mac_address);
            }
        }
    }
    Ok(())
}

async fn handle_vpn(connection: &ConnectionArgs, args: VpnArgs) -> anyhow::Result<()> {
    let client = connect_from_args(connection).await?;
    let service = VpnService::new(&client);
    match args.command {
        VpnCommand::Status => {
            let status = service.get_status().await?;
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        VpnCommand::Wireguard(wg) => match wg.command {
            WireGuardCommand::ListClients => {
                let peers = service.list_wireguard_peers().await?;
                println!("{}", serde_json::to_string_pretty(&peers)?);
            }
            WireGuardCommand::AddClient { name, public_key, server_id, tunnel } => {
                let mut peer = threatflux_unifi_sdk::WireGuardPeer::new(name, public_key);
                if let Some(server_id) = server_id {
                    peer.server_id = Some(server_id);
                }
                peer.allowed_ips = match tunnel {
                    TunnelMode::Full => vec!["0.0.0.0/0".to_string()],
                    TunnelMode::Split => vec![],
                };
                let created = service.create_wireguard_peer(&peer).await?;
                println!("{}", serde_json::to_string_pretty(&created)?);
            }
        },
    }
    Ok(())
}

async fn handle_firewall(connection: &ConnectionArgs, args: FirewallArgs) -> anyhow::Result<()> {
    let client = connect_from_args(connection).await?;
    let service = FirewallService::new(&client);
    match args.command {
        FirewallCommand::ListRules { ruleset } => {
            let rules = if let Some(ruleset) = ruleset {
                service.list_rules_by_ruleset(ruleset.into()).await?
            } else {
                service.list_rules().await?
            };
            println!("{}", serde_json::to_string_pretty(&rules)?);
        }
        FirewallCommand::AddRule { name, ruleset, action, src, dst } => {
            let mut rule =
                threatflux_unifi_sdk::FirewallRule::new(name, ruleset.into(), action.into());
            if let Some(src) = src {
                rule = rule.with_src_address(src);
            }
            if let Some(dst) = dst {
                rule = rule.with_dst_address(dst);
            }
            let created = service.create_rule(&rule).await?;
            println!("{}", serde_json::to_string_pretty(&created)?);
        }
    }
    Ok(())
}

async fn handle_port_forward(
    connection: &ConnectionArgs,
    args: PortForwardArgs,
) -> anyhow::Result<()> {
    let client = connect_from_args(connection).await?;
    let service = PortForwardService::new(&client);
    match args.command {
        PortForwardCommand::List => {
            let forwards = service.list().await?;
            println!("{}", serde_json::to_string_pretty(&forwards)?);
        }
        PortForwardCommand::Add { name, port, fwd, fwd_port, protocol } => {
            let forward = PortForward::new(name, port.to_string(), fwd, fwd_port.to_string())
                .with_protocol(protocol.into());
            let created = service.create(&forward).await?;
            println!("{}", serde_json::to_string_pretty(&created)?);
        }
    }
    Ok(())
}

async fn handle_traffic(connection: &ConnectionArgs, args: TrafficArgs) -> anyhow::Result<()> {
    let client = connect_from_args(connection).await?;
    let service = TrafficService::new(&client);
    match args.command {
        TrafficCommand::Stats { hours } => {
            let end = chrono::Utc::now().timestamp();
            let start = end - (i64::from(hours) * 3600);
            let stats = service.get_stats(Timeframe::Custom(start, end)).await?;
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
    }
    Ok(())
}
