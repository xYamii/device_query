extern crate macos_accessibility_client;

use keymap::Keycode;
use mouse_state::{MouseState, ScrollDelta};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, Ordering};
use std::thread;

#[allow(dead_code, non_camel_case_types)]
mod cg_ffi {
    use std::ffi::c_void;

    pub type CGEventTapProxy = *mut c_void;
    pub type CGEventRef = *mut c_void;
    pub type CFMachPortRef = *mut c_void;
    pub type CFRunLoopSourceRef = *mut c_void;
    pub type CFRunLoopRef = *mut c_void;
    pub type CFAllocatorRef = *const c_void;
    pub type CFStringRef = *const c_void;

    pub const K_CG_HID_EVENT_TAP: u32 = 0;
    pub const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
    pub const K_CG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;
    pub const K_CG_EVENT_SCROLL_WHEEL: u32 = 22;
    pub const K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFFFFFE;
    pub const K_CG_SCROLL_WHEEL_EVENT_DELTA_AXIS1: u32 = 11; // vertical
    pub const K_CG_SCROLL_WHEEL_EVENT_DELTA_AXIS2: u32 = 12; // horizontal

    pub type CGEventTapCallBack = unsafe extern "C" fn(
        proxy: CGEventTapProxy,
        event_type: u32,
        event: CGEventRef,
        user_info: *mut c_void,
    ) -> CGEventRef;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        pub fn CGEventTapCreate(
            tap: u32,
            place: u32,
            options: u32,
            events_of_interest: u64,
            callback: CGEventTapCallBack,
            user_info: *mut c_void,
        ) -> CFMachPortRef;
        pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
        pub fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        pub fn CFMachPortCreateRunLoopSource(
            allocator: CFAllocatorRef,
            port: CFMachPortRef,
            order: i64,
        ) -> CFRunLoopSourceRef;
        pub fn CFRunLoopGetCurrent() -> CFRunLoopRef;
        pub fn CFRunLoopAddSource(
            rl: CFRunLoopRef,
            source: CFRunLoopSourceRef,
            mode: CFStringRef,
        );
        pub fn CFRunLoopRun();
        pub static kCFRunLoopCommonModes: CFStringRef;
    }
}
use self::cg_ffi::*;

// Global scroll accumulators
static SCROLL_VERTICAL: AtomicI32 = AtomicI32::new(0);
static SCROLL_HORIZONTAL: AtomicI32 = AtomicI32::new(0);
static HOOK_INITIALIZED: AtomicBool = AtomicBool::new(false);
static EVENT_TAP: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

unsafe extern "C" fn scroll_callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    if event_type == K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT {
        // Re-enable the tap if macOS disabled it due to timeout
        let tap = EVENT_TAP.load(Ordering::Relaxed);
        if !tap.is_null() {
            CGEventTapEnable(tap, true);
        }
        return event;
    }
    if event_type == K_CG_EVENT_SCROLL_WHEEL {
        let vertical =
            CGEventGetIntegerValueField(event, K_CG_SCROLL_WHEEL_EVENT_DELTA_AXIS1) as i32;
        let horizontal =
            CGEventGetIntegerValueField(event, K_CG_SCROLL_WHEEL_EVENT_DELTA_AXIS2) as i32;
        SCROLL_VERTICAL.fetch_add(vertical, Ordering::Relaxed);
        SCROLL_HORIZONTAL.fetch_add(horizontal, Ordering::Relaxed);
    }
    event
}

fn init_scroll_hook() {
    if HOOK_INITIALIZED.swap(true, Ordering::Relaxed) {
        return;
    }

    thread::spawn(|| unsafe {
        let event_mask: u64 = 1 << K_CG_EVENT_SCROLL_WHEEL;
        let tap = CGEventTapCreate(
            K_CG_HID_EVENT_TAP,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_OPTION_LISTEN_ONLY,
            event_mask,
            scroll_callback,
            std::ptr::null_mut(),
        );
        if tap.is_null() {
            HOOK_INITIALIZED.store(false, Ordering::Relaxed);
            return;
        }
        EVENT_TAP.store(tap, Ordering::Relaxed);

        let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
        if source.is_null() {
            HOOK_INITIALIZED.store(false, Ordering::Relaxed);
            return;
        }
        let run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
        CFRunLoopRun();
    });

    // Give the hook thread time to initialize
    thread::sleep(std::time::Duration::from_millis(10));
}

#[derive(Debug, Clone)]
pub struct DeviceState;
const MAPPING: &[(readkey::Keycode, Keycode)] = &[
    (readkey::Keycode::_0, Keycode::Key0),
    (readkey::Keycode::_1, Keycode::Key1),
    (readkey::Keycode::_2, Keycode::Key2),
    (readkey::Keycode::_3, Keycode::Key3),
    (readkey::Keycode::_4, Keycode::Key4),
    (readkey::Keycode::_5, Keycode::Key5),
    (readkey::Keycode::_6, Keycode::Key6),
    (readkey::Keycode::_7, Keycode::Key7),
    (readkey::Keycode::_8, Keycode::Key8),
    (readkey::Keycode::_9, Keycode::Key9),
    (readkey::Keycode::A, Keycode::A),
    (readkey::Keycode::B, Keycode::B),
    (readkey::Keycode::C, Keycode::C),
    (readkey::Keycode::D, Keycode::D),
    (readkey::Keycode::E, Keycode::E),
    (readkey::Keycode::F, Keycode::F),
    (readkey::Keycode::G, Keycode::G),
    (readkey::Keycode::H, Keycode::H),
    (readkey::Keycode::I, Keycode::I),
    (readkey::Keycode::J, Keycode::J),
    (readkey::Keycode::K, Keycode::K),
    (readkey::Keycode::L, Keycode::L),
    (readkey::Keycode::M, Keycode::M),
    (readkey::Keycode::N, Keycode::N),
    (readkey::Keycode::O, Keycode::O),
    (readkey::Keycode::P, Keycode::P),
    (readkey::Keycode::Q, Keycode::Q),
    (readkey::Keycode::R, Keycode::R),
    (readkey::Keycode::S, Keycode::S),
    (readkey::Keycode::T, Keycode::T),
    (readkey::Keycode::U, Keycode::U),
    (readkey::Keycode::V, Keycode::V),
    (readkey::Keycode::W, Keycode::W),
    (readkey::Keycode::X, Keycode::X),
    (readkey::Keycode::Y, Keycode::Y),
    (readkey::Keycode::Z, Keycode::Z),
    (readkey::Keycode::F1, Keycode::F1),
    (readkey::Keycode::F2, Keycode::F2),
    (readkey::Keycode::F3, Keycode::F3),
    (readkey::Keycode::F4, Keycode::F4),
    (readkey::Keycode::F5, Keycode::F5),
    (readkey::Keycode::F6, Keycode::F6),
    (readkey::Keycode::F7, Keycode::F7),
    (readkey::Keycode::F8, Keycode::F8),
    (readkey::Keycode::F9, Keycode::F9),
    (readkey::Keycode::F10, Keycode::F10),
    (readkey::Keycode::F11, Keycode::F11),
    (readkey::Keycode::F12, Keycode::F12),
    (readkey::Keycode::F13, Keycode::F13),
    (readkey::Keycode::F14, Keycode::F14),
    (readkey::Keycode::F15, Keycode::F15),
    (readkey::Keycode::F16, Keycode::F16),
    (readkey::Keycode::F17, Keycode::F17),
    (readkey::Keycode::F18, Keycode::F18),
    (readkey::Keycode::F19, Keycode::F19),
    (readkey::Keycode::F20, Keycode::F20),
    (readkey::Keycode::Keypad0, Keycode::Numpad0),
    (readkey::Keycode::Keypad1, Keycode::Numpad1),
    (readkey::Keycode::Keypad2, Keycode::Numpad2),
    (readkey::Keycode::Keypad3, Keycode::Numpad3),
    (readkey::Keycode::Keypad4, Keycode::Numpad4),
    (readkey::Keycode::Keypad5, Keycode::Numpad5),
    (readkey::Keycode::Keypad6, Keycode::Numpad6),
    (readkey::Keycode::Keypad7, Keycode::Numpad7),
    (readkey::Keycode::Keypad8, Keycode::Numpad8),
    (readkey::Keycode::Keypad9, Keycode::Numpad9),
    (readkey::Keycode::KeypadPlus, Keycode::NumpadAdd),
    (readkey::Keycode::KeypadMinus, Keycode::NumpadSubtract),
    (readkey::Keycode::KeypadDivide, Keycode::NumpadDivide),
    (readkey::Keycode::KeypadMultiply, Keycode::NumpadMultiply),
    (readkey::Keycode::KeypadEquals, Keycode::NumpadEquals),
    (readkey::Keycode::KeypadEnter, Keycode::NumpadEnter),
    (readkey::Keycode::KeypadDecimal, Keycode::NumpadDecimal),
    (readkey::Keycode::Escape, Keycode::Escape),
    (readkey::Keycode::Space, Keycode::Space),
    (readkey::Keycode::Control, Keycode::LControl),
    (readkey::Keycode::RightControl, Keycode::RControl),
    (readkey::Keycode::Shift, Keycode::LShift),
    (readkey::Keycode::RightShift, Keycode::RShift),
    (readkey::Keycode::Option, Keycode::LOption),
    (readkey::Keycode::RightOption, Keycode::ROption),
    (readkey::Keycode::Command, Keycode::Command),
    (readkey::Keycode::RightCommand, Keycode::RCommand),
    (readkey::Keycode::Return, Keycode::Enter),
    (readkey::Keycode::Up, Keycode::Up),
    (readkey::Keycode::Down, Keycode::Down),
    (readkey::Keycode::Left, Keycode::Left),
    (readkey::Keycode::Right, Keycode::Right),
    (readkey::Keycode::Delete, Keycode::Backspace),
    (readkey::Keycode::CapsLock, Keycode::CapsLock),
    (readkey::Keycode::Tab, Keycode::Tab),
    (readkey::Keycode::Home, Keycode::Home),
    (readkey::Keycode::End, Keycode::End),
    (readkey::Keycode::PageUp, Keycode::PageUp),
    (readkey::Keycode::PageDown, Keycode::PageDown),
    (readkey::Keycode::Help, Keycode::Insert),
    (readkey::Keycode::ForwardDelete, Keycode::Delete),
    (readkey::Keycode::Grave, Keycode::Grave),
    (readkey::Keycode::Minus, Keycode::Minus),
    (readkey::Keycode::Equal, Keycode::Equal),
    (readkey::Keycode::LeftBracket, Keycode::LeftBracket),
    (readkey::Keycode::RightBracket, Keycode::RightBracket),
    (readkey::Keycode::Backslash, Keycode::BackSlash),
    (readkey::Keycode::Semicolon, Keycode::Semicolon),
    (readkey::Keycode::Quote, Keycode::Apostrophe),
    (readkey::Keycode::Comma, Keycode::Comma),
    (readkey::Keycode::Period, Keycode::Dot),
    (readkey::Keycode::Slash, Keycode::Slash),
];

impl DeviceState {
    pub fn new() -> DeviceState {
        // TODO: remove this
        assert!(
            has_accessibility(),
            "This app does not have Accessibility Permissions enabled and will not work"
        );

        init_scroll_hook();
        DeviceState {}
    }

    /// returns `None` if app doesn't accessibility permissions.
    pub fn checked_new() -> Option<DeviceState> {
        if has_accessibility() {
            init_scroll_hook();
            Some(DeviceState {})
        } else {
            None
        }
    }

    pub fn query_pointer(&self) -> MouseState {
        let (x, y) = readmouse::Mouse::location();
        let button_pressed = vec![
            false,
            readmouse::Mouse::Left.is_pressed(),
            readmouse::Mouse::Right.is_pressed(),
            readmouse::Mouse::Center.is_pressed(),
            false,
        ];

        // Read and reset scroll delta atomically
        let scroll_delta = ScrollDelta {
            vertical: SCROLL_VERTICAL.swap(0, Ordering::Relaxed),
            horizontal: SCROLL_HORIZONTAL.swap(0, Ordering::Relaxed),
        };

        MouseState {
            coords: (x as i32, y as i32),
            button_pressed,
            scroll_delta,
        }
    }

    pub fn query_keymap(&self) -> Vec<Keycode> {
        MAPPING
            .iter()
            .filter(|(from, _)| from.is_pressed())
            .map(|(_, to)| *to)
            .collect()
    }
}

/// Returns true if the Accessibility permissions necessary for this library to work are granted
/// to this process
///
/// If this returns false, the app can request them through the OS APIs, or the user can:
///   1. open the MacOS system preferences
///   2. go to Security -> Privacy
///   3. scroll down to Accessibility and unlock it
///   4. Add the app that is using device_query (such as your terminal) to the list
///
fn has_accessibility() -> bool {
    use self::macos_accessibility_client::accessibility::*;
    // Without prompting:
    // application_is_trusted()

    // With prompting:
    application_is_trusted_with_prompt()
}
