//! Prometheus metrics + lightweight rolling stats.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use prometheus::{Encoder, Gauge, Histogram, IntCounter, IntGauge, Registry, TextEncoder};
use serde::Serialize;

#[derive(Clone)]
pub struct Metrics {
    pub registry: Registry,
    pub messages_total: IntCounter,
    pub aircraft_current: IntGauge,
    pub decode_errors: IntCounter,
    pub beast_reconnects: IntCounter,
    pub message_rate: Gauge,
    pub max_range_nm: Gauge,
    pub decode_latency: Histogram,
    window: Arc<RateWindow>,
}

#[derive(Default)]
struct RateWindow {
    count: AtomicU64,
}

impl Metrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        let messages_total = IntCounter::new(
            "flyscanner_messages_total",
            "Total Mode-S messages processed",
        )?;
        let aircraft_current =
            IntGauge::new("flyscanner_aircraft_current", "Aircraft currently tracked")?;
        let decode_errors =
            IntCounter::new("flyscanner_decode_errors_total", "Decode failures")?;
        let beast_reconnects =
            IntCounter::new("flyscanner_beast_reconnects_total", "Beast TCP reconnects")?;
        let message_rate = Gauge::new("flyscanner_message_rate", "Messages per second (approx)")?;
        let max_range_nm = Gauge::new(
            "flyscanner_max_range_nm",
            "Max observed range in nautical miles",
        )?;
        let decode_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new("flyscanner_decode_seconds", "Decode latency")
                .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05]),
        )?;

        registry.register(Box::new(messages_total.clone()))?;
        registry.register(Box::new(aircraft_current.clone()))?;
        registry.register(Box::new(decode_errors.clone()))?;
        registry.register(Box::new(beast_reconnects.clone()))?;
        registry.register(Box::new(message_rate.clone()))?;
        registry.register(Box::new(max_range_nm.clone()))?;
        registry.register(Box::new(decode_latency.clone()))?;

        Ok(Self {
            registry,
            messages_total,
            aircraft_current,
            decode_errors,
            beast_reconnects,
            message_rate,
            max_range_nm,
            decode_latency,
            window: Arc::new(RateWindow::default()),
        })
    }

    pub fn observe_message(&self) {
        self.messages_total.inc();
        self.window.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn tick_rate(&self) {
        let n = self.window.count.swap(0, Ordering::Relaxed);
        self.message_rate.set(n as f64);
    }

    pub fn render_prometheus(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).ok();
        String::from_utf8_lossy_owned(buffer)
    }

    pub fn summary(&self) -> StatsSummary {
        StatsSummary {
            messages_total: self.messages_total.get(),
            aircraft_current: self.aircraft_current.get(),
            message_rate: self.message_rate.get(),
            max_range_nm: self.max_range_nm.get(),
            decode_errors: self.decode_errors.get(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsSummary {
    pub messages_total: u64,
    pub aircraft_current: i64,
    pub message_rate: f64,
    pub max_range_nm: f64,
    pub decode_errors: u64,
}

/// Rolling ring buffer of (timestamp, value) for charts.
#[derive(Clone, Debug, Serialize)]
pub struct TimeSeries {
    pub points: VecDeque<(f64, f64)>,
    capacity: usize,
}

impl TimeSeries {
    pub fn new(capacity: usize) -> Self {
        Self {
            points: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, ts: f64, value: f64) {
        if self.points.len() >= self.capacity {
            self.points.pop_front();
        }
        self.points.push_back((ts, value));
    }
}

#[expect(dead_code)]
pub struct TimedScope {
    start: Instant,
}

#[expect(dead_code)]
impl TimedScope {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// Haversine distance in nautical miles.
pub fn range_nm(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R_NM: f64 = 3440.065;
    let (lat1, lon1, lat2, lon2) = (
        lat1.to_radians(),
        lon1.to_radians(),
        lat2.to_radians(),
        lon2.to_radians(),
    );
    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    R_NM * c
}
