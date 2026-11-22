//! Protocol data models for real-time WebSocket communication between Android and Desktop.

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: &str = "1.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    #[serde(default = "default_pressure")]
    pub pressure: f32,
    #[serde(default)]
    pub timestamp: i64,
}

fn default_pressure() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stroke {
    pub stroke_id: String,
    pub tool: String,
    pub color: String,
    pub width: f32,
    pub points: Vec<Point>,
    #[serde(default)]
    pub is_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayGeometry {
    pub width: u32,
    pub height: u32,
    pub aspect_ratio: f32,
}

impl Default for DisplayGeometry {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            aspect_ratio: 16.0 / 9.0,
        }
    }
}

/// Generic container for incoming client messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "hello")]
    Hello {
        #[serde(default)]
        version: String,
        #[serde(default)]
        client_name: String,
        #[serde(default)]
        device_id: String,
        #[serde(default)]
        device_name: String,
        #[serde(default)]
        token: Option<String>,
    },
    #[serde(rename = "pair_request")]
    PairRequest {
        device_id: String,
        #[serde(default)]
        device_name: String,
        pin: String,
    },
    #[serde(rename = "auth")]
    Auth {
        token: String,
    },
    #[serde(rename = "mode_change")]
    ModeChange {
        mode: String,
    },
    #[serde(rename = "stroke_start")]
    StrokeStart {
        stroke_id: String,
        tool: String,
        color: String,
        width: f32,
        x: f32,
        y: f32,
        #[serde(default = "default_pressure")]
        pressure: f32,
        #[serde(default)]
        timestamp: i64,
    },
    #[serde(rename = "stroke_move")]
    StrokeMove {
        stroke_id: String,
        points: Vec<Point>,
    },
    #[serde(rename = "stroke_end")]
    StrokeEnd {
        stroke_id: String,
        x: f32,
        y: f32,
        #[serde(default = "default_pressure")]
        pressure: f32,
        #[serde(default)]
        timestamp: i64,
    },
    #[serde(rename = "undo")]
    Undo,
    #[serde(rename = "redo")]
    Redo,
    #[serde(rename = "clear")]
    Clear,
    #[serde(rename = "set_timer")]
    SetTimer {
        seconds: f32,
    },
    #[serde(rename = "laser_pointer")]
    LaserPointer {
        x: f32,
        y: f32,
        active: bool,
    },
    #[serde(rename = "cursor_move")]
    CursorMove {
        #[serde(default)]
        dx: f32,
        #[serde(default)]
        dy: f32,
        #[serde(default)]
        x: f32,
        #[serde(default)]
        y: f32,
        #[serde(default = "default_pressure")]
        pressure: f32,
    },
    #[serde(rename = "pointer_down")]
    PointerDown {
        button: u32,
        #[serde(default)]
        x: f32,
        #[serde(default)]
        y: f32,
        #[serde(default = "default_pressure")]
        pressure: f32,
    },
    #[serde(rename = "pointer_up")]
    PointerUp {
        button: u32,
        #[serde(default)]
        x: f32,
        #[serde(default)]
        y: f32,
    },
    #[serde(rename = "ping")]
    Ping {
        #[serde(default)]
        timestamp: i64,
    },
    #[serde(rename = "disconnect")]
    Disconnect,
    #[serde(other)]
    Unknown,
}

/// Outgoing responses from desktop companion to Android app
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerResponse {
    #[serde(rename = "hello_ack")]
    HelloAck {
        version: String,
        server_name: String,
        authenticated: bool,
        pairing_required: bool,
        display: DisplayGeometry,
    },
    #[serde(rename = "pair_ack")]
    PairAck {
        success: bool,
        token: Option<String>,
        message: Option<String>,
    },
    #[serde(rename = "auth_ack")]
    AuthAck {
        success: bool,
        message: Option<String>,
    },
    #[serde(rename = "undo_ack")]
    UndoAck,
    #[serde(rename = "redo_ack")]
    RedoAck,
    #[serde(rename = "clear_ack")]
    ClearAck,
    #[serde(rename = "timer_ack")]
    TimerAck {
        seconds: f32,
    },
    #[serde(rename = "pong")]
    Pong {
        timestamp: i64,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
    },
}
