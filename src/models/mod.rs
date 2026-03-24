//! Data models for UniFi API resources.

pub mod backup;
pub mod client;
pub mod device;
pub mod dhcp;
pub mod dns;
pub mod firewall;
pub mod network;
pub mod port_forward;
pub mod routing;
pub mod site;
pub mod traffic;
pub mod vpn;

pub use backup::*;
pub use client::*;
pub use device::*;
pub use dhcp::*;
pub use dns::*;
pub use firewall::*;
pub use network::*;
pub use port_forward::*;
pub use routing::*;
pub use site::*;
pub use traffic::*;
pub use vpn::*;
