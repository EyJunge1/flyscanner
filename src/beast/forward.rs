//! Outbound Beast TCP push (Ultrafeeder-style `adsb,host,port,beast_*_out`).

use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use super::reduce::{normalize_uuid, reduce_plus_handshake, BeastReducer, FeedMode};
use super::server::OutFrame;
use crate::state::now_secs;

/// Options for an outbound feed target.
#[derive(Debug, Clone)]
pub struct ForwardOpts {
    pub mode: FeedMode,
    pub uuid: Option<String>,
    /// Beast-reduce base interval seconds (default 0.5 like Ultrafeeder).
    pub reduce_interval: f64,
}

impl Default for ForwardOpts {
    fn default() -> Self {
        Self {
            mode: FeedMode::Beast,
            uuid: None,
            reduce_interval: 0.5,
        }
    }
}

/// Keep pushing frames to `host:port`, reconnecting on failure.
///
/// For `BeastReduce` / `BeastReducePlus`, frames are rate-limited client-side.
/// Plus sends the UUID handshake once per successful connect.
pub fn spawn_forwarder(
    host: String,
    port: u16,
    tx: broadcast::Sender<OutFrame>,
    opts: ForwardOpts,
    cancel: CancellationToken,
) {
    tokio::spawn(async move {
        let target = format!("{host}:{port}");
        let uuid = opts
            .mode
            .needs_uuid_handshake()
            .then(|| normalize_uuid(opts.uuid.as_deref()));
        loop {
            if cancel.is_cancelled() {
                return;
            }
            info!(%target, mode = ?opts.mode, "beast-forward connecting");
            match TcpStream::connect((host.as_str(), port)).await {
                Ok(mut stream) => {
                    info!(%target, mode = ?opts.mode, "beast-forward connected");
                    if let Some(ref u) = uuid {
                        if let Err(e) = stream.write_all(&reduce_plus_handshake(u)).await {
                            warn!(%target, error = %e, "beast-forward handshake failed");
                            tokio::select! {
                                () = cancel.cancelled() => return,
                                () = tokio::time::sleep(Duration::from_secs(3)) => {}
                            }
                            continue;
                        }
                        info!(%target, uuid = %u, "beast-reduce-plus handshake sent");
                    }

                    let mut rx = tx.subscribe();
                    let mut reducer = opts
                        .mode
                        .is_reduce()
                        .then(|| BeastReducer::new(opts.reduce_interval));
                    let mut last_expire = now_secs();

                    loop {
                        tokio::select! {
                            () = cancel.cancelled() => return,
                            msg = rx.recv() => match msg {
                                Ok(frame) => {
                                    if let Some(ref mut red) = reducer {
                                        let now = now_secs();
                                        if now - last_expire > 60.0 {
                                            red.expire(now, 120.0);
                                            last_expire = now;
                                        }
                                        if !red.accept(&frame.payload, now) {
                                            continue;
                                        }
                                    }
                                    if let Err(e) = stream.write_all(&frame.encode()).await {
                                        warn!(%target, error = %e, "beast-forward write failed");
                                        break;
                                    }
                                }
                                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                                Err(broadcast::error::RecvError::Closed) => return,
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!(%target, error = %e, "beast-forward connect failed");
                }
            }
            tokio::select! {
                () = cancel.cancelled() => return,
                () = tokio::time::sleep(Duration::from_secs(3)) => {}
            }
        }
    });
}
