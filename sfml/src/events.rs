use sfml::{
    graphics::RenderWindow,
    window::{Event, Key},
};

pub fn process_events(window: &mut RenderWindow) {
    while let Some(event) = window.poll_event() {
        match event {
            Event::Closed => window.close(),
            Event::KeyPressed {
                code: Key::Escape | Key::Q,
                ..
            } => window.close(),
            _ => {}
        }
        if event == Event::Closed {
            window.close();
        }
    }
}
