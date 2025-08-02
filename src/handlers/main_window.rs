use std::fs::{read, File};
use std::os::unix::thread;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc};

use once_cell::sync::Lazy;
use std::sync::Mutex;

use bincode::config::standard;
use bincode::serde::decode_from_slice;
use slint::{ComponentHandle, SharedString, Weak};

use crate::handlers::create_vault_window::CreateVaultWindowHandler;
use crate::handlers::{WindowHandler, dialog_window::DialogWindowHandler};
use crate::models::vault::{self, Vault};
use crate::MainWindow;
use crate::utils::file::read_encrypted_file;


static GLOBAL_VAULT: Lazy<Mutex<Option<Vault>>> = Lazy::new(|| Mutex::new(None));

/// Coordinates the MainWindow lifecycle and UI behavior.
/// Holds ownership to prevent premature drop and supports weak upgrade for event binding.
pub(crate) struct MainWindowHandler {
    _window_strong: MainWindow,  // Keeps the actual window alive with struct
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

    fn setup(handler: &Self) {
        let window = handler.get_window().upgrade().unwrap();
        
        let window_weak = window.as_weak();
        let create_vault_window_handler = CreateVaultWindowHandler::new();

        let window_weak_create = window_weak.clone();
        window.on_open_create_database(move || {
            Self::open_create_vault_window(&window_weak_create, &create_vault_window_handler);
        });

        let window_weak_open = window_weak.clone();
        window.on_open_unlock_vault(move || {
            if let Some(path) = Self::open_existing_vault() {
                let weak = window_weak_open.upgrade().unwrap();
                let path = SharedString::from(path.display().to_string());

                weak.set_vault_location(path);
            }
        });

        window.on_unlock_vault(move |location: SharedString, password: SharedString| {
            let path = PathBuf::from_str(location.as_str()).unwrap();
            if let Ok(bytes) = read_encrypted_file(&path, password.into()) {
                match decode_from_slice(&bytes, standard()) {
                    Ok((decoded_bytes, _bytes_read)) => {
                        let mut vault_guard = GLOBAL_VAULT.lock().unwrap();
                        *vault_guard = Some(decoded_bytes);
                        window_weak.upgrade().unwrap().set_vault_open(true);
                    },
                    Err(e) => {
                        rfd::MessageDialog::new()
                            .set_title("Decode Error")
                            .set_description(&format!("Failed to decode vault data: {}", e))
                            .set_buttons(rfd::MessageButtons::Ok)
                            .show();
                        return;
                    }
                }
            } else {
                rfd::MessageDialog::new()
                    .set_title("Error")
                    .set_description("Failed to open vault file. Check password.")
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show();
            }
        });
    }

    fn open_existing_vault() -> Option<PathBuf> {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select Vault File")
            .add_filter("Vault Files", &["vault"])
            .pick_file()
        {
            return Some(path);
        }

        None
    }

    /// Opens the CreateVaultWindow if it's not already visible and
    /// disables input on the main window while open.
    fn open_create_vault_window(window_weak: &Weak<MainWindow>, create_vault_window_handler: &Arc<Mutex<CreateVaultWindowHandler>>) {
        // TODO: Disable window input when another window is open

        if let Ok(mut handler) = create_vault_window_handler.lock() {
            if !handler.get_visible() {
                //window_weak.upgrade().unwrap().set_disable_input(true);
                handler.show();
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
                // Exit the entire program if main window is closed
                std::process::exit(0);
            });
        }
    }
}