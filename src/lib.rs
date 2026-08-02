#![allow(clippy::doc_markdown)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::future_not_send)]
//! # ThreatFlux UniFi SDK
//!
//! An async, community-maintained client for automating UniFi Network
//! controllers. This crate is not an official Ubiquiti or UniFi SDK and is not
//! affiliated with or endorsed by Ubiquiti Inc.
//!
//! It provides a low-level [`UnifiClient`], typed service modules, models, a
//! declarative configuration/sync engine, and the packaged `unifi-cli`
//! binary. Controller endpoints are partly undocumented or reverse-engineered
//! and can change independently of this crate.
//!
//! ## Security-sensitive TLS default
//!
//! [`UnifiConfig::new`] currently disables certificate verification for
//! compatibility with self-signed controllers. This is unsafe on an untrusted
//! network. Configure a certificate trusted by the client runtime and call
//! [`UnifiConfig::with_verify_ssl(true)`](UnifiConfig::with_verify_ssl) in
//! production. Changing the default requires a separate compatibility plan.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use std::env;
//!
//! use threatflux_unifi_sdk::{NetworkService, UnifiClient, UnifiConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = UnifiConfig::new(
//!         env::var("UNIFI_HOST")?,
//!         env::var("UNIFI_USERNAME")?,
//!         env::var("UNIFI_PASSWORD")?,
//!     )
//!     .with_site(env::var("UNIFI_SITE").unwrap_or_else(|_| "default".to_owned()))
//!     .with_verify_ssl(true)
//!     .with_timeout(30);
//!
//!     let client = UnifiClient::connect(config).await?;
//!     let networks = NetworkService::new(&client).list().await?;
//!     for network in networks {
//!         println!("{}", network.name);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! The environment lookup in this example is application code;
//! [`UnifiConfig`] does not read environment variables.
//!
//! ## Runtime boundaries
//!
//! - Controller-type detection is a login-endpoint heuristic, not version
//!   negotiation.
//! - The SDK has no automatic request replay or backoff policy.
//! - Typed GET/POST/PUT and raw GET paths return
//!   [`UnifiError::SessionExpired`] on 401 without replaying the failed
//!   request. A caller retry causes login before the next request. DELETE uses
//!   a separate error path and does not clear session state on 401.
//! - Service-name Cargo features are compatibility markers and do not currently
//!   gate modules or dependencies.

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
