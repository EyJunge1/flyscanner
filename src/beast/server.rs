use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use super::frame::BeastFrame;

/// Mode-S payload ready for Beast encode / reduce filtering.
#[derive(Debug, Clone)]
pub struct OutFrame {
    pub payload: Vec<u8>,
    pub signal_level: u8,
}

impl OutFrame {
    #[must_use]
    pub fn from_beast(frame: &BeastFrame) -> Option<Self> {
        frame.mode_s_payload().map(|p| Self {
            payload: p.to_vec(),
            signal_level: frame.signal_level,
        })
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        BeastFrame::encode_mode_s(&self.payload, self.signal_level)
    }
}

/// Beast output server: clients connect and receive forwarded Mode-S frames.
pub struct BeastOutServer {
    tx: broadcast::Sender<OutFrame>,
}

impl BeastOutServer {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn publisher(&self) -> broadcast::Sender<OutFrame> {
        self.tx.clone()
    }

    pub fn publish_frame(&self, frame: &BeastFrame) {
        if let Some(out) = OutFrame::from_beast(frame) {
            let _ = self.tx.send(out);
        }
    }

    pub async fn listen(
        self: Arc<Self>,
        addr: SocketAddr,
        cancel: CancellationToken,
    ) -> std::io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!(%addr, "beast-out listening");

        loop {
            tokio::select! {
                () = cancel.cancelled() => return Ok(()),
                accepted = listener.accept() => {
                    let (socket, peer) = accepted?;
                    info!(%peer, "beast-out client connected");
                    let mut rx = self.tx.subscribe();
                    let cancel = cancel.clone();
                    tokio::spawn(async move {
                        if let Err(e) = write_loop(socket, &mut rx, &cancel).await {
                            warn!(%peer, error = %e, "beast-out client disconnected");
                        }
                    });
                }
            }
        }
    }
}

async fn write_loop(
    mut socket: TcpStream,
    rx: &mut broadcast::Receiver<OutFrame>,
    cancel: &CancellationToken,
) -> std::io::Result<()> {
    loop {
        tokio::select! {
            () = cancel.cancelled() => break,
            msg = rx.recv() => match msg {
                Ok(frame) => {
                    socket.write_all(&frame.encode()).await?;
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    }
    Ok(())
}
