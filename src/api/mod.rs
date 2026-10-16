//! Axum HTTP + WebSocket API.

mod routes;
mod static_assets;
mod ws;

use std::net::SocketAddr;
use std::sync::Arc;

use crate::history::HistoryStore;
use crate::metrics::{Metrics, TimeSeries};
use crate::state::AircraftStore;
use axum::Router;
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub use routes::AppState;
pub use ws::WsMessage;

#[expect(dead_code)]
pub struct ApiConfig {
    pub listen: SocketAddr,
    pub receiver_lat: f64,
    pub receiver_lon: f64,
}

pub fn build_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(routes::router())
        .fallback(static_assets::static_handler)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub fn new_state(
    store: Arc<RwLock<AircraftStore>>,
    history: Option<Arc<HistoryStore>>,
    metrics: Arc<Metrics>,
    receiver_lat: f64,
    receiver_lon: f64,
) -> Arc<AppState> {
    let (tx, _) = broadcast::channel(512);
    Arc::new(AppState {
        store,
        history,
        metrics,
        ws_tx: tx,
        receiver_lat,
        receiver_lon,
        // ~2h @ 1 Hz — graphs1090-style live charts in the UI
        rate_series: Arc::new(RwLock::new(TimeSeries::new(7200))),
        aircraft_series: Arc::new(RwLock::new(TimeSeries::new(7200))),
        range_series: Arc::new(RwLock::new(TimeSeries::new(7200))),
    })
}
