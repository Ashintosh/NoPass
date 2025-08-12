use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use slint::{ComponentHandle, Weak};

use crate::DialogWindow;
use crate::errors::ui_errors::UiError;
use crate::handlers::WindowHandler;


pub(crate) struct _DialogWindowHandler {
    _window_strong: DialogWindow,
    window: Weak<DialogWindow>,
    visible: Arc<AtomicBool>,
}

impl _DialogWindowHandler {
    pub(crate) fn _new() -> Result<Self, UiError> {
        let window = DialogWindow::new().map_err(|_| UiError::_WindowCreation("Failed to create dialog window".into()))?;
        let weak = window.as_weak();
        let handler = Self {
            _window_strong: window,
            window: weak,
            visible: Arc::new(AtomicBool::new(false)),
        };

        //Self::setup(&handler);
        Ok(handler)
    }

    fn _setup(&self) {
        todo!()
    }
}

impl WindowHandler for _DialogWindowHandler {
    type Component = DialogWindow;

    fn get_window(&self) -> Weak<Self::Component> {
        self.window.clone()
    }

    fn get_visible(&self) -> bool {
        self.visible.load(Ordering::Relaxed)
    }

    fn get_visible_arc(&self) -> Arc<AtomicBool> {
        self.visible.clone()
    }

    fn set_visible(&self, value: bool) {
        self.visible.store(value, Ordering::Relaxed);
    }
}