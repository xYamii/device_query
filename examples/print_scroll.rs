extern crate device_query;

use device_query::{DeviceEvents, DeviceEventsHandler, MouseScrollEvent};
use std::thread;
use std::time::Duration;

fn main() {
    println!("Starting scroll test. Scroll your mouse wheel to see events.");
    println!("Press Ctrl+C to exit.\n");

    let event_handler = DeviceEventsHandler::new(Duration::from_millis(10))
        .expect("Could not initialize event loop");

    let _guard = event_handler.on_mouse_scroll(|event| {
        match event {
            MouseScrollEvent::VerticalUp => println!("Scroll Up"),
            MouseScrollEvent::VerticalDown => println!("Scroll Down"),
            MouseScrollEvent::HorizontalRight => println!("Scroll Right"),
            MouseScrollEvent::HorizontalLeft => println!("Scroll Left"),
        }
    });

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
