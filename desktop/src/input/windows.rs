//! Windows SendInput backend for absolute/relative mouse and pen injection.

use super::InputBackend;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_MOUSE, MOUSEINPUT,
    MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
};

pub struct WindowsInputBackend {
    width: u32,
    height: u32,
}

impl WindowsInputBackend {
    pub fn new() -> Option<Self> {
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN) as u32;
            let height = GetSystemMetrics(SM_CYSCREEN) as u32;
            if width == 0 || height == 0 {
                return None;
            }
            Some(Self {
                width: width.max(800),
                height: height.max(600),
            })
        }
    }
}

impl InputBackend for WindowsInputBackend {
    fn get_screen_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn move_absolute(&self, norm_x: f32, norm_y: f32, _pressure: f32) {
        unsafe {
            let clamped_x = norm_x.clamp(0.0, 1.0);
            let clamped_y = norm_y.clamp(0.0, 1.0);
            let dx = (clamped_x * 65535.0) as i32;
            let dy = (clamped_y * 65535.0) as i32;

            let mut input = std::mem::zeroed::<INPUT>();
            input.r#type = INPUT_MOUSE;
            input.Anonymous.mi = MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            };

            SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
        }
    }

    fn move_relative(&self, dx: f32, dy: f32, _pressure: f32) {
        unsafe {
            let mut input = std::mem::zeroed::<INPUT>();
            input.r#type = INPUT_MOUSE;
            input.Anonymous.mi = MOUSEINPUT {
                dx: dx as i32,
                dy: dy as i32,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            };

            SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
        }
    }

    fn button_down(&self, button: u32, _pressure: f32) {
        unsafe {
            let flag = match button {
                1 => MOUSEEVENTF_LEFTDOWN,
                2 => MOUSEEVENTF_MIDDLEDOWN,
                3 => MOUSEEVENTF_RIGHTDOWN,
                _ => MOUSEEVENTF_LEFTDOWN,
            };
            let mut input = std::mem::zeroed::<INPUT>();
            input.r#type = INPUT_MOUSE;
            input.Anonymous.mi = MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flag,
                time: 0,
                dwExtraInfo: 0,
            };
            SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
        }
    }

    fn button_up(&self, button: u32) {
        unsafe {
            let flag = match button {
                1 => MOUSEEVENTF_LEFTUP,
                2 => MOUSEEVENTF_MIDDLEUP,
                3 => MOUSEEVENTF_RIGHTUP,
                _ => MOUSEEVENTF_LEFTUP,
            };
            let mut input = std::mem::zeroed::<INPUT>();
            input.r#type = INPUT_MOUSE;
            input.Anonymous.mi = MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flag,
                time: 0,
                dwExtraInfo: 0,
            };
            SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
        }
    }

    fn cleanup(&self) {}
}
