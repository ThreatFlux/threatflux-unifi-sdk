//! UniFi device (AP, switch, gateway) models.

use serde::{Deserialize, Serialize};

use crate::types::DeviceType;

/// A UniFi network device (AP, switch, gateway, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Device MAC address (unique identifier).
    #[serde(rename = "mac")]
    pub mac_address: String,

    /// Device model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Device type.
    #[serde(rename = "type", default)]
    pub device_type: DeviceType,

    /// Device name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// IP address.
    #[serde(rename = "ip", skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// Firmware version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Whether the device is adopted.
    #[serde(default)]
    pub adopted: bool,

    /// Device state (1=connected, 0=disconnected).
    #[serde(default)]
    pub state: i32,

    /// Uptime in seconds.
    #[serde(default)]
    pub uptime: u64,

    /// Last seen timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<u64>,

    /// Whether upgradable.
    #[serde(default)]
    pub upgradable: bool,

    /// Available upgrade version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_to_firmware: Option<String>,

    /// Site ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,

    /// Serial number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,

    /// CPU utilization percentage.
    #[serde(rename = "system-stats.cpu", skip_serializing_if = "Option::is_none")]
    pub cpu_usage: Option<f32>,

    /// Memory utilization percentage.
    #[serde(rename = "system-stats.mem", skip_serializing_if = "Option::is_none")]
    pub mem_usage: Option<f32>,

    /// Number of connected clients (for APs).
    #[serde(rename = "num_sta", default)]
    pub num_clients: u32,

    /// Device ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// LED override setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub led_override: Option<String>,

    /// Whether the device is disabled.
    #[serde(default)]
    pub disabled: bool,

    /// Config network (mgmt VLAN, etc).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_network: Option<DeviceConfigNetwork>,

    /// Ethernet table (port configs).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ethernet_table: Vec<EthernetPort>,

    /// Port overrides.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub port_overrides: Vec<PortOverride>,
}

/// Device network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfigNetwork {
    /// IP type (dhcp, static).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub ip_type: Option<String>,

    /// Static IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,

    /// DNS servers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns1: Option<String>,

    /// DNS servers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns2: Option<String>,

    /// Gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,

    /// Netmask.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub netmask: Option<String>,
}

/// Ethernet port information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthernetPort {
    /// Port name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// MAC address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac: Option<String>,

    /// Port number.
    #[serde(default)]
    pub num_port: u32,
}

/// Port override configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortOverride {
    /// Port index.
    #[serde(default)]
    pub port_idx: u32,

    /// Port name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Port profile ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portconf_id: Option<String>,

    /// PoE mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poe_mode: Option<String>,

    /// Aggregation group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregate_num_ports: Option<u32>,
}

impl Device {
    /// Check if device is online.
    #[must_use]
    pub const fn is_online(&self) -> bool {
        self.state == 1
    }

    /// Check if device is an access point.
    #[must_use]
    pub const fn is_ap(&self) -> bool {
        matches!(self.device_type, DeviceType::Uap)
    }

    /// Check if device is a switch.
    #[must_use]
    pub const fn is_switch(&self) -> bool {
        matches!(self.device_type, DeviceType::Usw)
    }

    /// Check if device is a gateway.
    #[must_use]
    pub const fn is_gateway(&self) -> bool {
        matches!(
            self.device_type,
            DeviceType::Usg
                | DeviceType::Udm
                | DeviceType::UdmPro
                | DeviceType::UdmSe
                | DeviceType::Udr
                | DeviceType::Uxg
        )
    }

    /// Get display name.
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.mac_address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_device(device_type: DeviceType, state: i32) -> Device {
        Device {
            mac_address: "aa:bb:cc:dd:ee:ff".to_string(),
            model: None,
            device_type,
            name: None,
            ip_address: None,
            version: None,
            adopted: true,
            state,
            uptime: 0,
            last_seen: None,
            upgradable: false,
            upgrade_to_firmware: None,
            site_id: None,
            serial: None,
            cpu_usage: None,
            mem_usage: None,
            num_clients: 0,
            id: None,
            led_override: None,
            disabled: false,
            config_network: None,
            ethernet_table: vec![],
            port_overrides: vec![],
        }
    }

    #[test]
    fn test_device_is_online() {
        let online = make_device(DeviceType::Uap, 1);
        let offline = make_device(DeviceType::Uap, 0);
        assert!(online.is_online());
        assert!(!offline.is_online());
    }

    #[test]
    fn test_device_type_checks() {
        let ap = make_device(DeviceType::Uap, 1);
        let sw = make_device(DeviceType::Usw, 1);
        let gw = make_device(DeviceType::Usg, 1);
        let udm = make_device(DeviceType::Udm, 1);

        assert!(ap.is_ap());
        assert!(!ap.is_switch());

        assert!(sw.is_switch());
        assert!(!sw.is_ap());

        assert!(gw.is_gateway());
        assert!(udm.is_gateway());
    }
}
