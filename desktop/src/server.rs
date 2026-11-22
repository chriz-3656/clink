//! Asynchronous Tokio WebSocket server for Clink companion.

use crate::config::ClinkConfig;
use crate::input::InputBackend;
use crate::overlay::renderer::OverlayRenderer;
use crate::pairing::PairingManager;
use crate::protocol::{
    ClientMessage, DisplayGeometry, Point, ServerResponse, Stroke, PROTOCOL_VERSION,
};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, info, warn};

pub struct ClinkServer {
    config: ClinkConfig,
    pairing_mgr: PairingManager,
    renderer: Arc<Mutex<OverlayRenderer>>,
    input_backend: Arc<dyn InputBackend>,
    redraw_tx: broadcast::Sender<()>,
}

impl ClinkServer {
    pub fn new(
        config: ClinkConfig,
        pairing_mgr: PairingManager,
        renderer: Arc<Mutex<OverlayRenderer>>,
        input_backend: Arc<dyn InputBackend>,
        redraw_tx: broadcast::Sender<()>,
    ) -> Self {
        Self {
            config,
            pairing_mgr,
            renderer,
            input_backend,
            redraw_tx,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port).parse()?;
        let listener = TcpListener::bind(addr).await?;
        info!("Native Clink WebSocket server listening on ws://{}", addr);

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            debug!("Incoming connection from {}", peer_addr);

            let pairing_mgr = self.pairing_mgr.clone();
            let renderer = self.renderer.clone();
            let input_backend = self.input_backend.clone();
            let redraw_tx = self.redraw_tx.clone();
            let require_auth = self.config.require_auth;

            tokio::spawn(async move {
                if let Err(e) = handle_connection(
                    stream,
                    peer_addr,
                    pairing_mgr,
                    renderer,
                    input_backend,
                    redraw_tx,
                    require_auth,
                )
                .await
                {
                    debug!("Connection with {} closed: {}", peer_addr, e);
                }
            });
        }
    }
}

async fn handle_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
    pairing_mgr: PairingManager,
    renderer: Arc<Mutex<OverlayRenderer>>,
    input_backend: Arc<dyn InputBackend>,
    redraw_tx: broadcast::Sender<()>,
    require_auth: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut ws_stream = tokio_tungstenite::accept_async(stream).await?;
    info!("Client connected: {}", peer_addr);

    let mut authenticated = !require_auth;

    while let Some(msg_result) = ws_stream.next().await {
        let msg = msg_result?;
        if msg.is_close() {
            break;
        }

        if let Message::Text(text) = msg {
            let client_msg: ClientMessage = match serde_json::from_str(&text) {
                Ok(m) => m,
                Err(e) => {
                    warn!("Invalid message payload: {} (raw: {})", e, text);
                    continue;
                }
            };

            let response = process_message(
                client_msg,
                &pairing_mgr,
                &renderer,
                &*input_backend,
                &mut authenticated,
            );

            let _ = redraw_tx.send(());

            if let Some(resp) = response {
                let resp_text = serde_json::to_string(&resp)?;
                ws_stream.send(Message::Text(resp_text)).await?;
            }
        }
    }

    input_backend.cleanup();
    info!("Client disconnected: {}", peer_addr);
    Ok(())
}

fn process_message(
    msg: ClientMessage,
    pairing_mgr: &PairingManager,
    renderer: &Arc<Mutex<OverlayRenderer>>,
    input_backend: &dyn InputBackend,
    authenticated: &mut bool,
) -> Option<ServerResponse> {
    match msg {
        ClientMessage::Hello {
            version: _,
            client_name: _,
            device_id: _,
            device_name: _,
            token,
        } => {
            let is_auth = if let Some(ref t) = token {
                pairing_mgr.verify_token(t)
            } else {
                false
            };

            if is_auth {
                *authenticated = true;
            }

            let (w, h) = input_backend.get_screen_size();
            Some(ServerResponse::HelloAck {
                version: PROTOCOL_VERSION.to_string(),
                server_name: "clink-rust".to_string(),
                authenticated: *authenticated,
                pairing_required: !*authenticated,
                display: DisplayGeometry {
                    width: w,
                    height: h,
                    aspect_ratio: w as f32 / h.max(1) as f32,
                },
            })
        }
        ClientMessage::PairRequest {
            device_id,
            device_name: _,
            pin,
        } => {
            if let Some(token) = pairing_mgr.verify_pin_and_issue_token(&pin, &device_id) {
                *authenticated = true;
                info!("Device '{}' paired successfully", device_id);
                Some(ServerResponse::PairAck {
                    success: true,
                    token: Some(token),
                    message: Some("Paired successfully".to_string()),
                })
            } else {
                warn!("Invalid PIN '{}' submitted by device '{}'", pin, device_id);
                Some(ServerResponse::PairAck {
                    success: false,
                    token: None,
                    message: Some("Invalid PIN".to_string()),
                })
            }
        }
        ClientMessage::Auth { token } => {
            if pairing_mgr.verify_token(&token) {
                *authenticated = true;
                Some(ServerResponse::AuthAck {
                    success: true,
                    message: Some("Authenticated".to_string()),
                })
            } else {
                Some(ServerResponse::AuthAck {
                    success: false,
                    message: Some("Invalid or expired session token".to_string()),
                })
            }
        }
        ClientMessage::ModeChange { mode } => {
            debug!("Client switched mode to: {}", mode);
            None
        }
        ClientMessage::StrokeStart {
            stroke_id,
            tool,
            color,
            width,
            x,
            y,
            pressure,
            timestamp,
        } => {
            if *authenticated {
                let p = Point {
                    x,
                    y,
                    pressure,
                    timestamp,
                };
                let stroke = Stroke {
                    stroke_id,
                    tool,
                    color,
                    width,
                    points: vec![p],
                    is_completed: false,
                };
                if let Ok(mut r) = renderer.lock() {
                    r.start_stroke(stroke);
                }
            }
            None
        }
        ClientMessage::StrokeMove { stroke_id, points } => {
            if *authenticated {
                if let Ok(mut r) = renderer.lock() {
                    r.append_points(&stroke_id, points);
                }
            }
            None
        }
        ClientMessage::StrokeEnd {
            stroke_id,
            x,
            y,
            pressure,
            timestamp,
        } => {
            if *authenticated {
                let final_p = Point {
                    x,
                    y,
                    pressure,
                    timestamp,
                };
                if let Ok(mut r) = renderer.lock() {
                    r.end_stroke(&stroke_id, Some(final_p));
                }
            }
            None
        }
        ClientMessage::Undo => {
            if *authenticated {
                if let Ok(mut r) = renderer.lock() {
                    r.undo();
                }
                Some(ServerResponse::UndoAck)
            } else {
                None
            }
        }
        ClientMessage::Redo => {
            if *authenticated {
                if let Ok(mut r) = renderer.lock() {
                    r.redo();
                }
                Some(ServerResponse::RedoAck)
            } else {
                None
            }
        }
        ClientMessage::Clear => {
            if *authenticated {
                if let Ok(mut r) = renderer.lock() {
                    r.clear();
                }
                Some(ServerResponse::ClearAck)
            } else {
                None
            }
        }
        ClientMessage::SetTimer { seconds } => {
            if *authenticated {
                if let Ok(mut r) = renderer.lock() {
                    r.fade_timeout = seconds;
                }
                Some(ServerResponse::TimerAck { seconds })
            } else {
                None
            }
        }
        ClientMessage::LaserPointer { x, y, active } => {
            if *authenticated {
                if let Ok(mut r) = renderer.lock() {
                    r.set_laser(x, y, active);
                }
            }
            None
        }
        ClientMessage::CursorMove {
            dx,
            dy,
            x,
            y,
            pressure,
        } => {
            if *authenticated {
                if dx != 0.0 || dy != 0.0 {
                    input_backend.move_relative(dx, dy, pressure);
                } else {
                    input_backend.move_absolute(x, y, pressure);
                }
            }
            None
        }
        ClientMessage::PointerDown {
            button,
            x: _,
            y: _,
            pressure,
        } => {
            if *authenticated {
                input_backend.button_down(button, pressure);
            }
            None
        }
        ClientMessage::PointerUp {
            button,
            x: _,
            y: _,
        } => {
            if *authenticated {
                input_backend.button_up(button);
            }
            None
        }
        ClientMessage::Ping { timestamp } => Some(ServerResponse::Pong { timestamp }),
        ClientMessage::Disconnect => {
            input_backend.cleanup();
            None
        }
        ClientMessage::Unknown => None,
    }
}
