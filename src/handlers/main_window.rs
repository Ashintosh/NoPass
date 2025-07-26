use crate::handlers::dialog_window;
use crate::handlers::{WindowHandler, dialog_window::DialogWindowHandler};
use crate::errors::ui_errors::UiError;
use crate::MainWindow;

use slint::{ComponentHandle, Weak};
use std::sync::{Arc, Mutex};

/// Coordinates the MainWindow lifecycle and UI behavior.
/// Holds ownership to prevent premature drop and supports weak upgrade for event binding.
pub(crate) struct MainWindowHandler {
    _window_strong: MainWindow,
    window: Weak<MainWindow>,
    visible: Arc<Mutex<bool>>,
}

impl MainWindowHandler {
    /// Creates a new `MainWindowHandler` and sets up window behavior.
    /// Panics on window creation failure (app can't continue without it).
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

    /// Wires up dialog toggle and close behavior.
    /// I know this is bad, but it works.
    fn setup_toggle_dialog(&self) {
        if let Some(window) = self.get_window().upgrade() {
            let window_weak = window.as_weak();

            let dialog_window_handler = match DialogWindowHandler::new() {
                Ok(handler) => Arc::new(Mutex::new(handler)),
                Err(err) => {
                    eprintln!("{:?}", err);
                    return;
                }
            };

            // Cloned once per use site 
            let window_weak_for_toggle = window_weak.clone();
            let dialog_handler_for_toggle = dialog_window_handler.clone();

            window.on_toggle_dialog(move || {
                Self::bind_toggle_event(&window_weak_for_toggle, &dialog_handler_for_toggle);
            });

            // Separate weak clone needed here to avoid double mut borrow of `window`
            let window_weak = window.as_weak();
            Self::bind_dialog_close(&window_weak, dialog_window_handler);
        }
    }

    /// Shows or hides the dialog window and updates the `dialog_visible` flag accordingly.
    /// Assumes `window_weak` and dialog are valid at this point in execution.
    fn bind_toggle_event(window_weak: &Weak<MainWindow>, dialog_window_handler: &Arc<Mutex<DialogWindowHandler>>) {
        let Some(window) = window_weak.upgrade() else {
            eprintln!("Failed to upgrade MainWindow weak reference");
            return;
        };

        let Ok(mut dialog) = dialog_window_handler.lock() else {
            eprintln!("Failed to lock DialogWindowHandler mutex");
            return;
        };

        let Some(dialog_window) = dialog.get_window().upgrade() else {
            eprintln!("Dialog window was not initialized");
            return;
        };

        dialog_window.set_win_title("Dialog box".into());
        dialog_window.set_message("This is a dialog box".into());
        
        if window.get_dialog_visible() {
            window.set_dialog_visible(false);
            dialog.hide();
        } else {
            window.set_dialog_visible(true);
            dialog.show();
        }
    }

    /// Binds dialog close event to sync internal visibility state.
    /// Uses a separate clone for safe closure capture (avoids interior mutability issues).
    fn bind_dialog_close(window_weak: &Weak<MainWindow>, dialog_window_handler: Arc<Mutex<DialogWindowHandler>>) {

        let Ok(dialog) = dialog_window_handler.lock() else {
            eprintln!("Failed to lock DialogWindowHandler mutex in close binding");
            return;
        };

        let Some(dialog_window) = dialog.get_window().upgrade() else {
            eprintln!("Dialog window is not initialized");
            return;
        };

        let window_weak = window_weak.clone();
        let dialog_window_handler_clone = dialog_window_handler.clone();

        dialog_window.window().on_close_requested(move || {
            let Some(window) = window_weak.upgrade() else {
                eprintln!("Failed to upgrade MainWindow weak reference in close handler");
                return slint::CloseRequestResponse::HideWindow;
            };

            window.set_dialog_visible(false);

            if let Ok(mut dialog) = dialog_window_handler_clone.lock() {
                dialog.set_visible(false);
            } else {
                eprintln!("Failed to lock DialogWindowHandler mutex during clone event");
            }

            slint::CloseRequestResponse::HideWindow
        });
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
                // Exit the entire program if main window is closed
                std::process::exit(0);
            });
        }
    }
}