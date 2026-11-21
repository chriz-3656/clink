# Clink Communication Protocol Specification - Version 1.0

## 1. Overview
Clink employs an asynchronous, bi-directional WebSocket protocol for real-time communication between the Android client and the Linux desktop companion engine.
- Default Port: `8765`
- Supported Transports:
  - **USB Connection**: Host sets up `adb reverse tcp:8765 tcp:8765`. Android connects to `ws://127.0.0.1:8765`.
  - **Wi-Fi Connection**: Android connects to `ws://<LINUX_HOST_IP>:8765` over local wireless LAN.

## 2. Coordinate System
- All drawing canvas and screen coordinates are normalized floats `[0.0, 1.0]`, where `(0.0, 0.0)` is the top-left corner and `(1.0, 1.0)` is the bottom-right corner.
- Relative pointer movements transmit normalized delta `dx, dy` floats with acceleration modifiers applied on the companion engine.
- Stylus pressure values are normalized floats `[0.0, 1.0]`. When hardware pressure is unavailable, default is `1.0` during contact and `0.0` otherwise.

## 3. Handshake and Authentication
### 3.1 Client Hello (`hello`)
Client sends connection greeting and device metadata:
```json
{
  "type": "hello",
  "version": "1.0",
  "client_name": "clink-android",
  "device_id": "android_9876543210",
  "device_name": "Redmi A5",
  "token": "optional-pre-shared-token"
}
```

### 3.2 Server Hello Acknowledge (`hello_ack`)
Server responds with capabilities and display dimensions:
```json
{
  "type": "hello_ack",
  "version": "1.0",
  "server_name": "clink-linux",
  "authenticated": false,
  "pairing_required": true,
  "display": {
    "width": 1920,
    "height": 1080,
    "aspect_ratio": 1.777778
  }
}
```

### 3.3 Pairing Flow (`pair_request` & `pair_ack`)
If pairing is required, user authorizes the device (or enters a 4-6 digit numeric pairing PIN):
- Client Request:
```json
{
  "type": "pair_request",
  "device_id": "android_9876543210",
  "device_name": "Redmi A5",
  "pin": "482910"
}
```
- Server Response:
```json
{
  "type": "pair_ack",
  "success": true,
  "token": "a8f3b9c0d1e2f3a4b5c6d7e8f9a0b1c2"
}
```

### 3.4 Authentication (`auth` & `auth_ack`)
Client authenticates with the assigned token:
- Client:
```json
{
  "type": "auth",
  "token": "a8f3b9c0d1e2f3a4b5c6d7e8f9a0b1c2"
}
```
- Server:
```json
{
  "type": "auth_ack",
  "success": true,
  "message": "Authenticated successfully"
}
```

## 4. Operation Modes
- Mode A: `absolute_annotation` (Desktop transparent overlay)
- Mode B: `relative_tablet` (Graphics tablet pointer & button injection)

Mode switch message:
```json
{
  "type": "mode_change",
  "mode": "absolute_annotation"
}
```

## 5. Absolute Annotation Events

### 5.1 Stroke Start (`stroke_start`)
```json
{
  "type": "stroke_start",
  "stroke_id": "s_1716300000_1",
  "tool": "pen",
  "color": "#FF3366",
  "width": 4.0,
  "x": 0.2541,
  "y": 0.3125,
  "pressure": 0.75,
  "timestamp": 1716300000123
}
```
Supported `tool` values:
- `pen`
- `pencil`
- `highlighter`
- `eraser`
- `laser`
- `line`
- `arrow`
- `rectangle`
- `circle`

### 5.2 Stroke Move (`stroke_move`)
May be sent for single point or batched points to optimize latency:
```json
{
  "type": "stroke_move",
  "stroke_id": "s_1716300000_1",
  "points": [
    {"x": 0.2550, "y": 0.3130, "pressure": 0.76, "timestamp": 1716300000135},
    {"x": 0.2562, "y": 0.3142, "pressure": 0.78, "timestamp": 1716300000147}
  ]
}
```

### 5.3 Stroke End (`stroke_end`)
```json
{
  "type": "stroke_end",
  "stroke_id": "s_1716300000_1",
  "x": 0.2600,
  "y": 0.3200,
  "pressure": 0.0,
  "timestamp": 1716300000200
}
```

### 5.4 Canvas Actions
- Undo: `{"type": "undo"}`
- Redo: `{"type": "redo"}`
- Clear: `{"type": "clear"}`
- Laser Pointer Position:
```json
{
  "type": "laser_pointer",
  "x": 0.512,
  "y": 0.489,
  "active": true
}
```
- Overlay Toggle:
```json
{
  "type": "overlay_toggle",
  "visible": true
}
```

## 6. Relative Tablet Events
- Pointer movement:
```json
{
  "type": "cursor_move",
  "dx": 0.005,
  "dy": -0.002,
  "x": 0.450,
  "y": 0.620,
  "pressure": 0.65,
  "timestamp": 1716300000300
}
```
- Button down:
```json
{
  "type": "pointer_down",
  "button": 1,
  "x": 0.450,
  "y": 0.620,
  "pressure": 0.85,
  "timestamp": 1716300000310
}
```
- Button up:
```json
{
  "type": "pointer_up",
  "button": 1,
  "x": 0.450,
  "y": 0.620,
  "timestamp": 1716300000350
}
```

## 7. Heartbeat and Termination
- Ping: `{"type": "ping", "timestamp": 1716300000500}`
- Pong: `{"type": "pong", "timestamp": 1716300000500}`
- Disconnect: `{"type": "disconnect", "reason": "user_exit"}`
