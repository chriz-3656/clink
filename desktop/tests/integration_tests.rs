use clink_desktop::pairing::PairingManager;
use clink_desktop::protocol::{ClientMessage, Point, Stroke};
use clink_desktop::overlay::renderer::OverlayRenderer;

#[test]
fn test_pairing_workflow() {
    let mgr = PairingManager::new();
    let pin = mgr.get_pin();
    assert_eq!(pin.len(), 6);

    // Invalid PIN fails
    assert!(mgr.verify_pin_and_issue_token("000000", "test_device").is_none());

    // Valid PIN succeeds
    let token = mgr.verify_pin_and_issue_token(&pin, "test_device");
    assert!(token.is_some());
    let token_str = token.unwrap();
    assert!(mgr.verify_token(&token_str));

    // Token revocation
    mgr.revoke_token(&token_str);
    assert!(!mgr.verify_token(&token_str));
}

#[test]
fn test_protocol_serialization_roundtrip() {
    let stroke_msg = ClientMessage::StrokeStart {
        stroke_id: "s1".to_string(),
        tool: "pen".to_string(),
        color: "#FF3366".to_string(),
        width: 4.0,
        x: 0.5,
        y: 0.5,
        pressure: 0.8,
        timestamp: 1000,
    };

    let serialized = serde_json::to_string(&stroke_msg).expect("Serialize failed");
    assert!(serialized.contains("\"type\":\"stroke_start\""));
    assert!(serialized.contains("\"stroke_id\":\"s1\""));

    let deserialized: ClientMessage = serde_json::from_str(&serialized).expect("Deserialize failed");
    match deserialized {
        ClientMessage::StrokeStart { stroke_id, width, .. } => {
            assert_eq!(stroke_id, "s1");
            assert_eq!(width, 4.0);
        }
        _ => panic!("Unexpected deserialized variant"),
    }
}

#[test]
fn test_undo_redo_messages() {
    let undo_json = r#"{"type":"undo"}"#;
    let msg: ClientMessage = serde_json::from_str(undo_json).expect("Parse undo failed");
    assert!(matches!(msg, ClientMessage::Undo));

    let redo_json = r#"{"type":"redo"}"#;
    let msg: ClientMessage = serde_json::from_str(redo_json).expect("Parse redo failed");
    assert!(matches!(msg, ClientMessage::Redo));

    let clear_json = r#"{"type":"clear"}"#;
    let msg: ClientMessage = serde_json::from_str(clear_json).expect("Parse clear failed");
    assert!(matches!(msg, ClientMessage::Clear));
}

#[test]
fn test_renderer_undo_redo_lifecycle() {
    let mut renderer = OverlayRenderer::new(1920, 1080);
    assert_eq!(renderer.strokes.len(), 0);

    // 1. Draw stroke 1
    let s1 = Stroke {
        stroke_id: "s1".to_string(),
        tool: "pen".to_string(),
        color: "#FF3366".to_string(),
        width: 4.0,
        points: vec![Point { x: 0.1, y: 0.1, pressure: 1.0, timestamp: 0 }],
        is_completed: false,
    };
    renderer.start_stroke(s1);
    renderer.end_stroke("s1", Some(Point { x: 0.2, y: 0.2, pressure: 1.0, timestamp: 1 }));
    assert_eq!(renderer.strokes.len(), 1);

    // 2. Draw stroke 2
    let s2 = Stroke {
        stroke_id: "s2".to_string(),
        tool: "pen".to_string(),
        color: "#00E5FF".to_string(),
        width: 5.0,
        points: vec![Point { x: 0.3, y: 0.3, pressure: 1.0, timestamp: 2 }],
        is_completed: false,
    };
    renderer.start_stroke(s2);
    renderer.end_stroke("s2", Some(Point { x: 0.4, y: 0.4, pressure: 1.0, timestamp: 3 }));
    assert_eq!(renderer.strokes.len(), 2);

    // 3. Test UNDO
    assert!(renderer.undo());
    assert_eq!(renderer.strokes.len(), 1);
    assert_eq!(renderer.strokes[0].stroke.stroke_id, "s1");

    // 4. Test REDO
    assert!(renderer.redo());
    assert_eq!(renderer.strokes.len(), 2);
    assert_eq!(renderer.strokes[1].stroke.stroke_id, "s2");

    // 5. Test CLEAR
    renderer.clear();
    assert_eq!(renderer.strokes.len(), 0);

    // 6. Test UNDO after CLEAR restores canvas
    assert!(renderer.undo());
    assert_eq!(renderer.strokes.len(), 2);
}

#[test]
fn test_renderer_auto_fade_timer() {
    let mut renderer = OverlayRenderer::new(1920, 1080);
    renderer.fade_timeout = 0.05; // 50ms timeout for test

    let s1 = Stroke {
        stroke_id: "fade_test".to_string(),
        tool: "pen".to_string(),
        color: "#FF3366".to_string(),
        width: 4.0,
        points: vec![Point { x: 0.1, y: 0.1, pressure: 1.0, timestamp: 0 }],
        is_completed: false,
    };
    renderer.start_stroke(s1);
    renderer.end_stroke("fade_test", None);
    assert_eq!(renderer.strokes.len(), 1);

    // Initially not faded
    assert!(!renderer.tick_fade());
    assert_eq!(renderer.strokes.len(), 1);

    // Sleep past fade timeout
    std::thread::sleep(std::time::Duration::from_millis(60));
    assert!(renderer.tick_fade());
    assert_eq!(renderer.strokes.len(), 0);
}
