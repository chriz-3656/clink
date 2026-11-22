//! PIN and session token pairing manager.

use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PairingManager {
    inner: Arc<Mutex<PairingState>>,
}

#[derive(Debug)]
struct PairingState {
    current_pin: String,
    valid_tokens: HashSet<String>,
}

impl PairingManager {
    pub fn new() -> Self {
        let pin = format!("{:06}", rand::thread_rng().gen_range(100000..=999999));
        Self {
            inner: Arc::new(Mutex::new(PairingState {
                current_pin: pin,
                valid_tokens: HashSet::new(),
            })),
        }
    }

    pub fn get_pin(&self) -> String {
        let state = self.inner.lock().unwrap();
        state.current_pin.clone()
    }

    #[allow(dead_code)]
    pub fn regenerate_pin(&self) -> String {
        let mut state = self.inner.lock().unwrap();
        let new_pin = format!("{:06}", rand::thread_rng().gen_range(100000..=999999));
        state.current_pin = new_pin.clone();
        new_pin
    }

    pub fn verify_pin_and_issue_token(&self, pin: &str, device_id: &str) -> Option<String> {
        let mut state = self.inner.lock().unwrap();
        if state.current_pin == pin {
            // Generate deterministic yet secure token
            let raw_key = format!("{}:{}:{}", device_id, pin, Uuid::new_v4());
            let mut hasher = Sha256::new();
            hasher.update(raw_key.as_bytes());
            let token = hex::encode(hasher.finalize());
            state.valid_tokens.insert(token.clone());
            Some(token)
        } else {
            None
        }
    }

    pub fn verify_token(&self, token: &str) -> bool {
        let state = self.inner.lock().unwrap();
        state.valid_tokens.contains(token)
    }

    #[allow(dead_code)]
    pub fn revoke_token(&self, token: &str) {
        let mut state = self.inner.lock().unwrap();
        state.valid_tokens.remove(token);
    }
}
