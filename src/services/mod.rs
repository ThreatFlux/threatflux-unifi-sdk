//! Service modules for interacting with UniFi API resources.

pub mod backup;
pub mod clients;
pub mod devices;
pub mod dhcp;
pub mod dns;
pub mod firewall;
pub mod networks;
pub mod port_forward;
pub mod routing;
pub mod site;
pub mod traffic;
pub mod vpn;

pub use backup::BackupService;
pub use clients::ClientService;
pub use devices::DeviceService;
pub use dhcp::DhcpService;
pub use dns::DnsService;
pub use firewall::FirewallService;
pub use networks::NetworkService;
pub use port_forward::PortForwardService;
pub use routing::RoutingService;
pub use site::SiteService;
pub use traffic::TrafficService;
pub use vpn::VpnService;
