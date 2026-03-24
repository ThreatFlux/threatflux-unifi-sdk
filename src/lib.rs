#![allow(clippy::doc_markdown)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::future_not_send)]
//! # ThreatFlux UniFi SDK
//!
//! Native Rust SDK for UDM Pro and UniFi OS device automation including firewall rules,
//! port forwarding, VLANs, traffic management, VPN configuration, and network monitoring.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use threatflux_unifi_sdk::{UnifiClient, UnifiConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to UDM Pro
//!     let config = UnifiConfig::new("192.168.1.1", "admin", "password");
//!     let client = UnifiClient::connect(config).await?;
//!
//!     // List all networks
//!     let networks: Vec<serde_json::Value> = client.get("rest/networkconf").await?;
//!     println!("Found {} networks", networks.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **Authentication**: Session-based auth with CSRF token handling
//! - **Multi-site**: Support for multiple sites on the same controller
//! - **Controller detection**: Automatic detection of UniFi OS vs Classic controllers
//! - **TLS**: Handles self-signed certificates (configurable)
//!
//! ## Feature Flags
//!
//! - `full` (default): Enable all service modules
//! - `firewall`: Firewall rules and groups management
//! - `vpn`: WireGuard and site-to-site VPN configuration
//! - `traffic`: Traffic rules and DPI statistics
//! - `clients`: Client management and blocking
//! - `devices`: Device adoption and configuration
//! - `dhcp`: DHCP reservations and leases
//! - `dns`: Local DNS records and filtering

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod services;
pub mod sync;
pub mod types;

pub use client::{ControllerType, UnifiClient, UnifiConfig};
pub use config::{UnifiConnectionConfig, UnifiDeclarativeConfig, load_config};
pub use error::{ApiResponse, ApiResponseMeta, Result, UnifiError};
pub use models::{
    Backup, BackupSettings, Client, ClientGroup, ClientStats, Device, DhcpLease, DhcpReservation,
    DnsRecord, FirewallGroup, FirewallRule, Network, PortForward, RouteTableEntry, Site, SiteStats,
    SiteVpn, StaticRoute, SystemInfo, TrafficRule, WireGuardPeer, WireGuardServer,
};
pub use services::{
    BackupService, ClientService, DeviceService, DhcpService, DnsService, FirewallService,
    NetworkService, PortForwardService, RoutingService, SiteService, TrafficService, VpnService,
};
pub use sync::{SyncAction, SyncChange, SyncOptions, SyncPlan, SyncReport};
pub use types::*;
