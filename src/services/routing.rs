//! Routing management service.

use tracing::instrument;

use crate::client::UnifiClient;
use crate::error::{Result, UnifiError};
use crate::models::routing::{RouteTableEntry, StaticRoute};

/// Service for managing routing.
pub struct RoutingService<'a> {
    client: &'a UnifiClient,
}

impl<'a> RoutingService<'a> {
    /// Create a new routing service.
    #[must_use]
    pub const fn new(client: &'a UnifiClient) -> Self {
        Self { client }
    }

    /// List all static routes.
    #[instrument(skip(self))]
    pub async fn list_static_routes(&self) -> Result<Vec<StaticRoute>> {
        self.client.get("rest/routing").await
    }

    /// Get a static route by ID.
    #[instrument(skip(self))]
    pub async fn get_static_route(&self, id: &str) -> Result<StaticRoute> {
        let routes: Vec<StaticRoute> = self.client.get(&format!("rest/routing/{id}")).await?;
        routes
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::NotFound(format!("Route {id} not found")))
    }

    /// Create a static route.
    #[instrument(skip(self, route))]
    pub async fn create_static_route(&self, route: &StaticRoute) -> Result<StaticRoute> {
        let routes: Vec<StaticRoute> = self.client.post("rest/routing", route).await?;
        routes
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No route returned".to_string()))
    }

    /// Update a static route.
    #[instrument(skip(self, route))]
    pub async fn update_static_route(&self, id: &str, route: &StaticRoute) -> Result<StaticRoute> {
        let routes: Vec<StaticRoute> =
            self.client.put(&format!("rest/routing/{id}"), route).await?;
        routes
            .into_iter()
            .next()
            .ok_or_else(|| UnifiError::InvalidResponse("No route returned".to_string()))
    }

    /// Delete a static route.
    #[instrument(skip(self))]
    pub async fn delete_static_route(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("rest/routing/{id}")).await
    }

    /// Enable or disable a static route.
    #[instrument(skip(self))]
    pub async fn set_route_enabled(&self, id: &str, enabled: bool) -> Result<StaticRoute> {
        let mut route = self.get_static_route(id).await?;
        route.enabled = enabled;
        self.update_static_route(id, &route).await
    }

    /// Create a route to a network via a gateway.
    #[instrument(skip(self))]
    pub async fn add_route_via_gateway(
        &self,
        name: &str,
        network: &str,
        gateway: &str,
    ) -> Result<StaticRoute> {
        let route = StaticRoute::via_gateway(network, gateway).with_name(name);
        self.create_static_route(&route).await
    }

    /// Create a blackhole route (drop traffic).
    #[instrument(skip(self))]
    pub async fn add_blackhole_route(&self, name: &str, network: &str) -> Result<StaticRoute> {
        let route = StaticRoute::blackhole(network).with_name(name);
        self.create_static_route(&route).await
    }

    /// Get the routing table from a gateway device.
    #[instrument(skip(self))]
    pub async fn get_route_table(&self, device_mac: &str) -> Result<Vec<RouteTableEntry>> {
        let cmd = serde_json::json!({
            "cmd": "show-route-table",
            "mac": device_mac
        });
        self.client.command("devmgr", &cmd).await
    }

    /// Find routes to a specific network.
    #[instrument(skip(self))]
    pub async fn find_routes_for_network(&self, network: &str) -> Result<Vec<StaticRoute>> {
        let routes = self.list_static_routes().await?;
        Ok(routes.into_iter().filter(|r| r.network == network).collect())
    }

    /// Get routes by gateway.
    #[instrument(skip(self))]
    pub async fn find_routes_via_gateway(&self, gateway: &str) -> Result<Vec<StaticRoute>> {
        let routes = self.list_static_routes().await?;
        Ok(routes.into_iter().filter(|r| r.gateway == gateway).collect())
    }
}
