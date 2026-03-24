//! DHCP management service.

use tracing::instrument;

use crate::client::UnifiClient;
use crate::error::{Result, UnifiError};
use crate::models::dhcp::{DhcpLease, DhcpReservation};

/// Service for managing DHCP reservations and leases.
pub struct DhcpService<'a> {
    client: &'a UnifiClient,
}

impl<'a> DhcpService<'a> {
    /// Create a new DHCP service.
    #[must_use]
    pub const fn new(client: &'a UnifiClient) -> Self {
        Self { client }
    }

    /// List all DHCP reservations (fixed IP assignments).
    #[instrument(skip(self))]
    pub async fn list_reservations(&self) -> Result<Vec<DhcpReservation>> {
        self.client.get("rest/user").await
    }

    /// Get reservations for a specific network.
    #[instrument(skip(self))]
    pub async fn list_reservations_by_network(
        &self,
        network_id: &str,
    ) -> Result<Vec<DhcpReservation>> {
        let reservations = self.list_reservations().await?;
        Ok(reservations
            .into_iter()
            .filter(|r| r.network_id.as_deref() == Some(network_id))
            .collect())
    }

    /// Get a reservation by MAC address.
    #[instrument(skip(self))]
    pub async fn get_reservation(&self, mac: &str) -> Result<DhcpReservation> {
        let reservations = self.list_reservations().await?;
        reservations
            .into_iter()
            .find(|r| r.mac_address == mac)
            .ok_or_else(|| UnifiError::NotFound(format!("Reservation for {mac} not found")))
    }

    /// Create a DHCP reservation.
    #[instrument(skip(self, reservation))]
    pub async fn create_reservation(
        &self,
        reservation: &DhcpReservation,
    ) -> Result<DhcpReservation> {
        let payload = serde_json::json!({
            "mac": reservation.mac_address,
            "fixed_ip": reservation.ip_address,
            "name": reservation.name,
            "network_id": reservation.network_id,
            "use_fixedip": true
        });
        let result: Vec<DhcpReservation> = self.client.post("rest/user", &payload).await?;
        result
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No reservation returned".to_string()))
    }

    /// Create a simple DHCP reservation.
    #[instrument(skip(self))]
    pub async fn reserve_ip(
        &self,
        mac: &str,
        ip: &str,
        name: Option<&str>,
        network_id: Option<&str>,
    ) -> Result<DhcpReservation> {
        let mut reservation = DhcpReservation::new(mac, ip);
        if let Some(n) = name {
            reservation = reservation.with_name(n);
        }
        if let Some(nid) = network_id {
            reservation = reservation.with_network(nid);
        }
        self.create_reservation(&reservation).await
    }

    /// Update a DHCP reservation.
    #[instrument(skip(self))]
    pub async fn update_reservation(&self, mac: &str, ip: &str) -> Result<()> {
        let payload = serde_json::json!({
            "mac": mac,
            "fixed_ip": ip,
            "use_fixedip": true
        });
        let _: serde_json::Value = self.client.put(&format!("rest/user/{mac}"), &payload).await?;
        Ok(())
    }

    /// Delete a DHCP reservation (release fixed IP).
    #[instrument(skip(self))]
    pub async fn delete_reservation(&self, mac: &str) -> Result<()> {
        let payload = serde_json::json!({
            "mac": mac,
            "use_fixedip": false
        });
        let _: serde_json::Value = self.client.put(&format!("rest/user/{mac}"), &payload).await?;
        Ok(())
    }

    /// List active DHCP leases.
    #[instrument(skip(self))]
    pub async fn list_leases(&self) -> Result<Vec<DhcpLease>> {
        self.client.get("stat/sta").await
    }

    /// Find lease by MAC address.
    #[instrument(skip(self))]
    pub async fn get_lease(&self, mac: &str) -> Result<Option<DhcpLease>> {
        let leases = self.list_leases().await?;
        Ok(leases.into_iter().find(|l| l.mac_address == mac))
    }

    /// Find lease by IP address.
    #[instrument(skip(self))]
    pub async fn get_lease_by_ip(&self, ip: &str) -> Result<Option<DhcpLease>> {
        let leases = self.list_leases().await?;
        Ok(leases.into_iter().find(|l| l.ip_address == ip))
    }
}
