use std::sync::Arc;

use crate::history::HistoryStore;
use crate::metrics::{Metrics, TimeSeries};
use crate::state::{Aircraft, AircraftStore, TrackPoint};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use tracing::warn;

use super::ws::{ws_handler, WsMessage};

pub struct AppState {
    pub store: Arc<RwLock<AircraftStore>>,
    pub history: Option<Arc<HistoryStore>>,
    pub metrics: Arc<Metrics>,
    pub ws_tx: broadcast::Sender<WsMessage>,
    pub receiver_lat: f64,
    pub receiver_lon: f64,
    pub rate_series: Arc<RwLock<TimeSeries>>,
    pub aircraft_series: Arc<RwLock<TimeSeries>>,
    pub range_series: Arc<RwLock<TimeSeries>>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/aircraft", get(list_aircraft))
        .route("/api/aircraft/{icao}", get(get_aircraft))
        .route("/api/aircraft/{icao}/track", get(get_track))
        .route("/api/history/ptracks", get(ptracks))
        .route("/api/history/heatmap", get(heatmap))
        .route("/api/history/replay", get(replay))
        .route("/api/history/trace/{icao}", get(day_trace))
        .route("/api/stats", get(stats))
        .route("/api/stats/series", get(stats_series))
        .route("/api/config", get(config))
        .route("/metrics", get(prometheus))
        .route("/ws", get(ws_handler))
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok", "service": "flyscanner" }))
}

#[derive(Deserialize)]
pub struct FilterQuery {
    pub min_alt: Option<i32>,
    pub max_alt: Option<i32>,
    pub icao: Option<String>,
    pub callsign: Option<String>,
    /// `lat_south,lat_north,lon_west,lon_east`
    #[serde(rename = "box")]
    pub bbox: Option<String>,
    /// `lat,lon,radius_nm`
    pub circle: Option<String>,
    /// `lat,lon,radius_nm` — same as circle, sorted by distance, optional `limit`
    pub closest: Option<String>,
    pub limit: Option<usize>,
}

async fn list_aircraft(
    State(state): State<Arc<AppState>>,
    Query(q): Query<FilterQuery>,
) -> Json<Vec<Aircraft>> {
    let store = state.store.read().await;
    let mut list = store.list_owned();
    if let Some(min) = q.min_alt {
        list.retain(|a| a.altitude.is_some_and(|x| x >= min));
    }
    if let Some(max) = q.max_alt {
        list.retain(|a| a.altitude.is_some_and(|x| x <= max));
    }
    if let Some(icao) = q.icao {
        let icao = icao.to_lowercase();
        list.retain(|a| a.icao24.to_hex().contains(&icao));
    }
    if let Some(cs) = q.callsign {
        let cs = cs.to_uppercase();
        list.retain(|a| {
            a.callsign
                .as_ref()
                .is_some_and(|c| c.to_uppercase().contains(&cs))
        });
    }

    if let Some(bbox) = q.bbox.as_deref()
        && let Some((lat_s, lat_n, lon_w, lon_e)) = parse_box(bbox)
    {
        list.retain(|a| {
            a.latitude.is_some_and(|lat| lat >= lat_s && lat <= lat_n)
                && a.longitude.is_some_and(|lon| lon >= lon_w && lon <= lon_e)
        });
    }

    if let Some(circle) = q.circle.as_deref().or(q.closest.as_deref())
        && let Some((clat, clon, radius_nm)) = parse_circle(circle)
    {
        list.retain(|a| match (a.latitude, a.longitude) {
            (Some(lat), Some(lon)) => {
                crate::metrics::range_nm(clat, clon, lat, lon) <= radius_nm
            }
            _ => false,
        });
        if q.closest.is_some() {
            list.sort_by(|a, b| {
                let da = a.latitude.zip(a.longitude).map_or(f64::MAX, |(lat, lon)| {
                    crate::metrics::range_nm(clat, clon, lat, lon)
                });
                let db = b.latitude.zip(b.longitude).map_or(f64::MAX, |(lat, lon)| {
                    crate::metrics::range_nm(clat, clon, lat, lon)
                });
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }

    if let Some(limit) = q.limit {
        list.truncate(limit);
    }
    Json(list)
}

fn parse_box(s: &str) -> Option<(f64, f64, f64, f64)> {
    let mut parts = s.split(',').filter_map(|p| p.trim().parse::<f64>().ok());
    Some((parts.next()?, parts.next()?, parts.next()?, parts.next()?))
}

fn parse_circle(s: &str) -> Option<(f64, f64, f64)> {
    let mut parts = s.split(',').filter_map(|p| p.trim().parse::<f64>().ok());
    Some((parts.next()?, parts.next()?, parts.next()?))
}

async fn get_aircraft(
    State(state): State<Arc<AppState>>,
    Path(icao): Path<String>,
) -> Result<Json<Aircraft>, StatusCode> {
    let store = state.store.read().await;
    store
        .get_str(&icao)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
pub struct TrackQuery {
    pub since: Option<f64>,
    pub until: Option<f64>,
    /// UTC calendar day `YYYY-MM-DD` (showTrace parity)
    pub day: Option<String>,
}

async fn get_track(
    State(state): State<Arc<AppState>>,
    Path(icao): Path<String>,
    Query(q): Query<TrackQuery>,
) -> Result<Json<Vec<TrackPoint>>, StatusCode> {
    if let Some(day) = &q.day {
        return day_trace_inner(&state, &icao, day).await;
    }

    // Prefer live track; fall back to history DB
    {
        let store = state.store.read().await;
        if let Some(ac) = store.get_str(&icao) {
            let mut pts = ac.track_history;
            if let Some(since) = q.since {
                pts.retain(|p| p.ts >= since);
            }
            if let Some(until) = q.until {
                pts.retain(|p| p.ts <= until);
            }
            if !pts.is_empty() {
                return Ok(Json(pts));
            }
        }
    }

    if let Some(hist) = &state.history {
        let now = crate::state::now_secs();
        let since = q.since.unwrap_or(now - 3600.0);
        let until = q.until.unwrap_or(now);
        return hist
            .trace(&icao, since, until)
            .await
            .map(Json)
            .map_err(|e| {
                warn!(error = %e, "history trace failed");
                StatusCode::INTERNAL_SERVER_ERROR
            });
    }

    Ok(Json(vec![]))
}

#[derive(Deserialize)]
pub struct DayTraceQuery {
    /// UTC calendar day `YYYY-MM-DD`
    pub day: String,
}

async fn day_trace(
    State(state): State<Arc<AppState>>,
    Path(icao): Path<String>,
    Query(q): Query<DayTraceQuery>,
) -> Result<Json<Vec<TrackPoint>>, StatusCode> {
    day_trace_inner(&state, &icao, &q.day).await
}

async fn day_trace_inner(
    state: &AppState,
    icao: &str,
    day: &str,
) -> Result<Json<Vec<TrackPoint>>, StatusCode> {
    let (since, until) = utc_day_bounds(day).ok_or(StatusCode::BAD_REQUEST)?;
    if let Some(hist) = &state.history {
        return hist
            .trace(icao, since, until)
            .await
            .map(Json)
            .map_err(|e| {
                warn!(error = %e, "history day-trace failed");
                StatusCode::INTERNAL_SERVER_ERROR
            });
    }
    Ok(Json(vec![]))
}

/// Parse `YYYY-MM-DD` as UTC midnight .. next midnight (unix seconds).
fn utc_day_bounds(day: &str) -> Option<(f64, f64)> {
    let mut parts = day.split('-');
    let y: i32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let date = chrono::NaiveDate::from_ymd_opt(y, m, d)?;
    let start = date.and_hms_opt(0, 0, 0)?.and_utc().timestamp() as f64;
    let end = (date + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)?
        .and_utc()
        .timestamp() as f64;
    Some((start, end))
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub hours: Option<f64>,
    pub interval: Option<f64>,
    pub max: Option<i64>,
}

async fn ptracks(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<Vec<TrackPoint>>, StatusCode> {
    let hours = q.hours.unwrap_or(8.0);
    let interval = q.interval.unwrap_or(15.0);
    let now = crate::state::now_secs();
    let since = now - hours * 3600.0;

    if let Some(hist) = &state.history {
        return hist
            .ptracks(since, now, interval)
            .await
            .map(Json)
            .map_err(|e| {
                warn!(error = %e, "history ptracks failed");
                StatusCode::INTERNAL_SERVER_ERROR
            });
    }

    let store = state.store.read().await;
    Ok(Json(store.heatmap_points(50_000)))
}

async fn heatmap(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<Vec<TrackPoint>>, StatusCode> {
    let hours = q.hours.unwrap_or(24.0);
    let max = q.max.unwrap_or(32_000);
    let now = crate::state::now_secs();
    let since = now - hours * 3600.0;

    if let Some(hist) = &state.history {
        return hist
            .heatmap(since, now, max)
            .await
            .map(Json)
            .map_err(|e| {
                warn!(error = %e, "history heatmap failed");
                StatusCode::INTERNAL_SERVER_ERROR
            });
    }

    let store = state.store.read().await;
    Ok(Json(store.heatmap_points(max as usize)))
}

async fn replay(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let hours = q.hours.unwrap_or(1.0);
    let now = crate::state::now_secs();
    let since = now - hours * 3600.0;

    if let Some(hist) = &state.history {
        let snaps = hist.replay_snapshots(since, now).await.map_err(|e| {
            warn!(error = %e, "history replay failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        return Ok(Json(serde_json::json!({ "snapshots": snaps })));
    }

    Ok(Json(serde_json::json!({ "snapshots": [] })))
}

async fn stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let s = state.metrics.summary();
    Json(serde_json::json!(s))
}

async fn stats_series(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let rate = state.rate_series.read().await.clone();
    let aircraft = state.aircraft_series.read().await.clone();
    let range = state.range_series.read().await.clone();
    Json(serde_json::json!({
        "message_rate": rate.points,
        "aircraft_count": aircraft.points,
        "max_range_nm": range.points,
    }))
}

#[derive(Serialize)]
struct ConfigResponse {
    receiver_lat: f64,
    receiver_lon: f64,
}

async fn config(State(state): State<Arc<AppState>>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        receiver_lat: state.receiver_lat,
        receiver_lon: state.receiver_lon,
    })
}

async fn prometheus(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4",
        )],
        state.metrics.render_prometheus(),
    )
}
