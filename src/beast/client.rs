use std::sync::Arc;
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use super::frame::{BeastFrame, FrameParser};
use crate::metrics::Metrics;

#[derive(Debug, Clone)]
pub struct BeastSource {
    pub host: String,
    pub port: u16,
}

impl BeastSource {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Reconnecting Beast TCP client. Yields decoded frames on a channel.
pub struct BeastClient;

impl BeastClient {
    /// Spawn a background task that reconnects until `cancel` fires.
    pub fn spawn(
        source: BeastSource,
        tx: mpsc::Sender<BeastFrame>,
        metrics: Option<Arc<Metrics>>,
        cancel: CancellationToken,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut backoff = Duration::from_secs(1);
            let max_backoff = Duration::from_secs(30);
            let mut ever_connected = false;

            loop {
                if cancel.is_cancelled() {
                    break;
                }
                match TcpStream::connect(source.addr()).await {
                    Ok(stream) => {
                        info!(addr = %source.addr(), "beast connected");
                        ever_connected = true;
                        backoff = Duration::from_secs(1);
                        if let Err(e) = Self::read_loop(stream, &tx, &cancel).await {
                            warn!(error = %e, "beast read loop ended");
                        }
                    }
                    Err(e) => {
                        warn!(
                            addr = %source.addr(),
                            error = %e,
                            "beast connect failed, retrying in {:?}",
                            backoff
                        );
                    }
                }
                if cancel.is_cancelled() {
                    break;
                }
                if ever_connected
                    && let Some(m) = &metrics
                {
                    m.beast_reconnects.inc();
                }
                tokio::select! {
                    () = cancel.cancelled() => break,
                    () = sleep(backoff) => {}
                }
                backoff = (backoff * 2).min(max_backoff);
            }
        })
    }

    async fn read_loop(
        mut stream: TcpStream,
        tx: &mpsc::Sender<BeastFrame>,
        cancel: &CancellationToken,
    ) -> std::io::Result<()> {
        let mut parser = FrameParser::new();
        let mut buf = [0u8; 4096];
        let read_timeout = Duration::from_mins(5);

        loop {
            let n = tokio::select! {
                () = cancel.cancelled() => return Ok(()),
                res = timeout(read_timeout, stream.read(&mut buf)) => match res {
                    Ok(Ok(0)) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "peer closed",
                        ));
                    }
                    Ok(Ok(n)) => n,
                    Ok(Err(e)) => return Err(e),
                    Err(_) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "no data for 300s",
                        ));
                    }
                },
            };

            let Some(chunk) = buf.get(..n) else {
                continue;
            };
            parser.push(chunk);
            for frame in parser.drain_frames() {
                if tx.send(frame).await.is_err() {
                    return Ok(());
                }
            }
        }
    }

    /// Convenience: connect once and return a stream of frames (no reconnect).
    #[expect(dead_code)]
    pub async fn connect_once(source: &BeastSource) -> std::io::Result<mpsc::Receiver<BeastFrame>> {
        let (tx, rx) = mpsc::channel(1024);
        let stream = TcpStream::connect(source.addr()).await?;
        let tx2 = tx.clone();
        let cancel = CancellationToken::new();
        tokio::spawn(async move {
            let _ = Self::read_loop(stream, &tx2, &cancel).await;
        });
        drop(tx);
        Ok(rx)
    }
}
