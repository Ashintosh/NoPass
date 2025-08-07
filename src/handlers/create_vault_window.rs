use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use bincode::config::standard;
use bincode::serde::encode_to_vec;
use slint::{ComponentHandle, SharedString, Weak};

use crate::CreateVaultWindow;
use crate::handlers::WindowHandler;
use crate::models::vault::Vault;
use crate::utils::crypto::Crypto;
use crate::utils::file;


/// Coordinates the MainWindow lifecycle and UI behavior.
/// Holds ownership to prevent premature drop and supports weak upgrade for event binding.
pub(crate) struct CreateVaultWindowHandler {
    _window_strong: CreateVaultWindow,
    window: Weak<CreateVaultWindow>,
    visible: Arc<Mutex<bool>>,
}

impl CreateVaultWindowHandler {
    /// Creates a new `MainWindowHandler` and sets up window behavior.
    /// Panics on window creation failure (app can't continue without it).
    pub(crate) fn new() -> Arc<Mutex<Self>> {
        let window = CreateVaultWindow::new().expect("Failed to create new MainWindow");
        let weak = window.as_weak();
        let handler = Self {
            _window_strong: window,
            window: weak,
            visible: Arc::new(Mutex::new(false)),
        };

        let handler = Arc::new(Mutex::new(handler));
        Self::setup(&handler);
        
        handler
    }

    fn setup(handler_arc: &Arc<Mutex<Self>>) {
        let handler_arc_clone = Arc::clone(handler_arc);
        let window = handler_arc_clone.lock().unwrap().get_window().upgrade().unwrap();
        //let window_weak = window.as_weak();

        let handler_arc_clone_done = Arc::clone(handler_arc);
        window.on_create_database_done(move |password: SharedString| {
            if let Some(vault_path) = Self::save_file_dialog() {
                Self::create_vault_file(&vault_path, password.into());

                if let Ok(mut handler) = handler_arc_clone_done.lock() {
                    handler.hide();
                }
            }
        }); 

        let handler_arc_clone_cancel = Arc::clone(handler_arc);
        window.on_create_database_cancel(move || {
            if let Ok(mut handler) = handler_arc_clone_cancel.lock() {
                handler.hide();
            }
        });
    }

    /// Create a new encrypted vault file at the specified path.
    /// Shows a confirmation or error dialog depending on success.
    fn create_vault_file(path: &PathBuf, password: String) {
        let vault = Vault::new();
        let encoded_vault = encode_to_vec(&vault, standard()).unwrap();
        let key = Crypto::derive_argon_key(password.as_bytes(), None).unwrap();

        match file::write_encrypted_file(&encoded_vault, path, &key) {
            Ok(()) => {
                let path = path.display().to_string();
                std::thread::spawn(move || {
                    rfd::MessageDialog::new()
                        .set_title("Vault Created")
                        .set_description(format!("Vault has been saved at {}", path))
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                });
            },
            Err(err) => {
                let message = 
                    if cfg!(debug_assertions) { err.as_str().to_string() } 
                    else { "Failed to create vault file.".to_string() };

                std::thread::spawn(move || {
                    rfd::MessageDialog::new()
                        .set_title("Error")
                        .set_description(message)
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                });
                
            }
        }
    }

    /// Opens a save file dialog and returns the user-selected path (if any).
    fn save_file_dialog() -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Select Vault Location")
            .set_file_name("passwords.vault")
            .save_file()
    }
}

impl WindowHandler for CreateVaultWindowHandler {
    type Component = CreateVaultWindow;

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