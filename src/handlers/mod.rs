pub(super) mod dialog_window;
pub(super) mod main_window;
pub(super) mod create_vault_window;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use slint::{ComponentHandle, Weak};

use crate::errors::ui_errors::UiError;
use crate::ui_error;


pub(super) trait WindowHandler {
    type Component: ComponentHandle;

    fn get_window(&self) -> Weak<Self::Component>;
    fn get_visible(&self) -> bool;
    fn get_visible_arc(&self) -> Arc<AtomicBool>;
    fn set_visible(&self, value: bool);
    
    fn initialize(&mut self) {
        if let Some(window) = self.get_window().upgrade() {
            let visible = self.get_visible_arc();
            window.window().on_close_requested( move || {
                if visible.load(Ordering::Relaxed) {
                    visible.store(false, Ordering::Relaxed);
                }
                slint::CloseRequestResponse::HideWindow
            });
        }
    }

    fn run(&mut self) -> Result<(), UiError> {
        if let Some(window) = self.get_window().upgrade() {
            self.initialize();
            self.set_visible(true);

            window.run()
                .map_err(|e| ui_error!(PlatformError, e, "Failed to run window"))?;
        }

        Err(ui_error!(Generic, "Failed to upgrade weak window"))
    }

    fn show(&mut self) -> Result<(), UiError> {
        if self.get_visible() {
            return Ok(());
        }
        
        if let Some(window) = self.get_window().upgrade() {
            self.initialize();
            self.set_visible(true);

            window.show()
                .map_err(|e| ui_error!(PlatformError, e, "Failed to show window"))?;
        }

        Err(ui_error!(Generic, "Failed to upgrade weak window"))
    }

    fn hide(&mut self) -> Result<(), UiError> {
        if let Some(window) = self.get_window().upgrade() {
            self.set_visible(false);

            window.hide()
                .map_err(|e| ui_error!(PlatformError, e, "Failed to hide window"))?;
        }

        Err(ui_error!(Generic, "Failed to upgrade weak window"))
    }

    fn cleanup() {
        unimplemented!()
    }
}

