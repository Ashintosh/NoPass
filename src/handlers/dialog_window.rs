use std::sync::{Arc, Mutex};

use crate::handlers::WindowHandler;
use crate::DialogWindow;

use slint::{ComponentHandle, Weak};

pub(crate) struct DialogWindowHandler {
    _window_strong: DialogWindow,
    window: Weak<DialogWindow>,
    visible: Arc<Mutex<bool>>,
}

impl DialogWindowHandler {
    pub(crate) fn new() -> Self {
        let window = DialogWindow::new().expect("Failed to create new DialogWindow");
        let weak = window.as_weak();
        let handler = Self {
            _window_strong: window,
            window: weak,
            visible: Arc::new(Mutex::new(false)),
        };

        //Self::setup(&handler);
        handler
    }

    fn setup(&self) {
        todo!()
    }
}

impl WindowHandler for DialogWindowHandler {
    type Component = DialogWindow;

    fn get_window(&self) -> Weak<Self::Component> {
        self.window.clone()
    }

    fn get_visible(&self) -> bool {
        if let Ok(visible) = self.visible.lock() {
            return *visible;
        }
        false
    }

    fn get_visible_arc(&self) -> Arc<Mutex<bool>> {
        self.visible.clone()
    }

    fn set_visible(&mut self, value: bool) {
        if let Ok(mut visible) = self.visible.lock() {
            *visible = value;
        }
    }
}