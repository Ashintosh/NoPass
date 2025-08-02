pub(super) mod dialog_window;
pub(super) mod main_window;
pub(super) mod create_vault_window;

use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, Weak};


pub(super) trait WindowHandler {
    type Component: ComponentHandle;

    fn get_window(&self) -> Weak<Self::Component>;
    fn get_visible(&self) -> bool;
    fn get_visible_arc(&self) -> Arc<Mutex<bool>>;
    fn set_visible(&mut self, value: bool);
    

    fn initialize(&mut self) {
        if let Some(window) = self.get_window().upgrade() {
            let visible = self.get_visible_arc();

            window.window().on_close_requested(move || {
                if let Ok(mut visible) = visible.lock() {
                    *visible = false;    
                }
                slint::CloseRequestResponse::HideWindow
            });
        }
    }

    fn run(&mut self) {
        if let Some(window) = self.get_window().upgrade() {
            self.initialize();
            self.set_visible(true);
            window.run().expect("Failed to run window");
        }
    }

    fn show(&mut self) {
        if self.get_visible() {
            return;
        }
        
        if let Some(window) = self.get_window().upgrade() {
            self.initialize();
            self.set_visible(true);
            window.show().expect("Failed to show window");
        }
    }

    fn hide(&mut self) {
        if let Some(window) = self.get_window().upgrade() {
            self.set_visible(false);
            window.hide().expect("Failed to hide window");
        }
    }
}

