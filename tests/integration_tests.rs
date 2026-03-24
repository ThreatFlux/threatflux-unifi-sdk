//! `UniFi` SDK integration tests.
//!
//! These tests require a running `UniFi` Network application. Configure with:
//! `UNIFI_HOST`, `UNIFI_USERNAME`, `UNIFI_PASSWORD`, optionally `UNIFI_SITE`,
//! `UNIFI_VERIFY_SSL`, `UNIFI_TIMEOUT_SECS`.

#[path = "integration/config.rs"]
mod config;
#[path = "integration/firewall.rs"]
mod firewall;
#[path = "integration/network.rs"]
mod network;
#[path = "integration/smoke.rs"]
mod smoke;
