use crate::handlers::{WindowHandler, dialog_window::DialogWindowHandler};
use crate::MainWindow;

use slint::{ComponentHandle, Weak};
use std::sync::{Arc, Mutex};

pub(crate) struct MainWindowHandler {
    _window_strong: MainWindow,
    window: Weak<MainWindow>,
    visible: Arc<Mutex<bool>>,
}

impl MainWindowHandler {
    pub(crate) fn new() -> Self {
        let window = MainWindow::new().expect("Failed to create new MainWindow");
        let weak = window.as_weak();
        let handler = Self {
            _window_strong: window,
            window: weak,
            visible: Arc::new(Mutex::new(false)),
        };

        Self::setup(&handler);
        handler
    }

    fn setup(&self) {
        self.setup_toggle_dialog();
    }

    fn setup_toggle_dialog(&self) {
        if let Some(window) = self.get_window().upgrade() {
            let dialog_window_handler = Arc::new(Mutex::new(DialogWindowHandler::new()));
            let dialog_window_handler_clone = dialog_window_handler.clone();

            let window_weak = window.as_weak();

            window.on_toggle_dialog(move || {
                let window = window_weak.upgrade().unwrap();
                let mut dialog = dialog_window_handler_clone.lock().unwrap();

                if window.get_dialog_visible() {
                    window.set_dialog_visible(false);
                    dialog.hide();
                } else {
                    window.set_dialog_visible(true);
                    dialog.show();
                }
            });

            let window_weak = window.as_weak();
            let dialog_handler_clone = dialog_window_handler.clone();

            if let Ok(dialog) = dialog_window_handler.lock() {
                dialog.get_window().upgrade().unwrap().window().on_close_requested(move || {
                    let window = window_weak.upgrade().unwrap();
                    window.set_dialog_visible(false);

                    if let Ok(mut dialog) = dialog_handler_clone.lock() {
                        dialog.set_visible(false);
                    }
                    slint::CloseRequestResponse::HideWindow
                });
            }
        }
    }
}

impl WindowHandler for MainWindowHandler {
    type Component = MainWindow;

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

    fn initialize(&mut self) {
        if let Some(window) = self.get_window().upgrade() {
            window.window().on_close_requested(move || {
                std::process::exit(0);
            });
        }
    }
}