use std::env::var_os;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use once_cell::sync::Lazy;
use rfd::MessageButtons;
use slint::{ComponentHandle, SharedString, Weak, VecModel, ModelRc};
use zeroize::Zeroize;

use crate::errors::ui_errors::UiError;
use crate::errors::app_errors::AppError;
use crate::handlers::create_vault_window::CreateVaultWindowHandler;
use crate::handlers::WindowHandler;
use crate::models::vault::{Item, Vault};
use crate::{app_error, ui_error, utils};
use crate::utils::file::{self, read_encrypted_file};
use crate::utils::zerobyte::ZeroByte;
use crate::{MainWindow, MainWindowItem, VaultItem};


/// Global static vault data, shared between handlers.
static GLOBAL_VAULT: Lazy<Mutex<Option<Vault>>> = Lazy::new(|| Mutex::new(None));

/// Coordinates the MainWindow lifecycle and UI behavior.
/// Holds ownership to prevent premature drop and supports weak upgrade for event binding.
pub(crate) struct MainWindowHandler {
    _window_strong: MainWindow,  // Keeps the actual window alive with struct
    window: Weak<MainWindow>,
    visible: Arc<AtomicBool>,
}

impl MainWindowHandler {
    /// Creates a new `MainWindowHandler` and sets up window behavior.
    pub(crate) async fn new() -> Result<Self, UiError> {
        let window = MainWindow::new()
            .map_err(|e| ui_error!(PlatformError, e, "Failed to create new MainWindow"))?;

        let weak = window.as_weak();
        let handler = Self {
            _window_strong: window,
            window: weak,
            visible: Arc::new(AtomicBool::new(false)),
        };

        Self::setup(&handler).await;
        Ok(handler)
    }

    async fn setup(handler: &Self) -> Result<(), UiError> {
        let window = handler.get_window().upgrade()
            .ok_or_else(|| ui_error!(Generic, "Failed to upgrade weak MainWindow"))?;

        let window_weak = window.as_weak();
        
        // This must be declared outside of the event handler to prevent creating a new window handler each time
        let create_vault_window_handler = CreateVaultWindowHandler::new().await;

        // Open create vault
        let window_weak_create = window_weak.clone();
        window.on_open_create_database(move || {
            Self::open_create_vault_window(&window_weak_create, &create_vault_window_handler);
        });

        // Open unlock vault
        let window_weak_open = window_weak.clone();
        window.on_open_unlock_vault(move || {
            Self::open_unlock_vault(&window_weak_open);
        });

        // Unlock vault
        let window_weak_unlock = window_weak.clone();
        window.on_unlock_vault(move |location: SharedString, password: SharedString| {
            let password = ZeroByte::from_shared_string(password);
            Self::unlock_vault(&window_weak_unlock, location.to_string(), &password);
        });

        // Load item
        let window_weak_load = window_weak.clone();
        window.on_load_selected_item(move |item_id: i32| {
            Self::load_selected_item(&window_weak_load, item_id);
        });

        // Save item
        let window_weak_save = window_weak.clone();
        window.on_save_selected_item(move |new_item: VaultItem| {
            Self::save_selected_item(&window_weak_save, new_item);
        });

        // Add item
        let window_weak_add = window_weak.clone();
        window.on_add_vault_item(move || {
            Self::add_vault_item(&window_weak_add);
        });

        // Delete item
        let window_weak_delete = window_weak.clone();
        window.on_delete_vault_item(move |item_id: i32| {
            if item_id >= 0 {
                Self::delete_vault_item(&window_weak_delete, item_id);
            } 
        });

        // Copy to clipboard
        window.on_copy_to_clipboard(move |text: SharedString| {
            utils::copy_text_to_clipboard(text.to_string());
        });

        Ok(())
    }

    /// Removed a vault item by ID and updates UI and state
    fn delete_vault_item(window: &Weak<MainWindow>, item_id: i32) -> Result<(), AppError> {
        {
            let mut vault_guard = GLOBAL_VAULT.lock()
                .map_err(|e| app_error!(PoisedState, e.to_string(), "Failed to get lock on GLOBAL_VAULT"))?;

            if let Some(vault) = vault_guard.as_mut() {
                if let Some(pos) = vault.items.iter().position(|item| item.id == item_id) {
                    vault.items.remove(pos);
                }
            }
        }

        if let Some(window) = window.upgrade() {
            Self::update_vault_items(&window);
        }

        Self::save_vault_state(window);
        Ok(())
    }

    /// Adds a new blank vault item with incremented ID and focuses on it
    fn add_vault_item(window: &Weak<MainWindow>) -> Result<(), AppError> {
        let new_id: i32;

        {
            let mut vault_guard = GLOBAL_VAULT.lock()
                .map_err(|e| app_error!(PoisedState, e.to_string(), "Failed to get lock on GLOBAL_VAULT"))?;

            if let Some(vault) = vault_guard.as_mut() {
                new_id = vault.nonce;
                vault.items.push(
                    Item { 
                        id: new_id,
                        name: "New Item".into(),
                        username: ZeroByte::with_capacity(0),
                        password: ZeroByte::with_capacity(0),
                        url: ZeroByte::with_capacity(0),
                        notes: ZeroByte::with_capacity(0),
                    }
                ); 

                vault.nonce += 1;
            }
        }

        Self::update_vault_items(&window.upgrade().unwrap());

        if let Some(window) = window.upgrade() {
            Self::update_vault_items(&window);
        }

        Self::save_vault_state(window);
        Ok(())
    }

    /// Encrypts and writes the vault to file
    fn save_vault_state(window: &Weak<MainWindow>) -> Result<(), AppError> {
        let window = window.upgrade()
            .unwrap_or_else(|| { app_error!(Generic, "Failed to upgrade weak window") });

        let mut vault_guard = GLOBAL_VAULT.lock()
            .map_err(|e| app_error!(PoisedState, e.to_string(), "Failed to get lock on GLOBAL_VAULT"))?;
        
        let mut vault = vault_guard.as_mut()
            .unwrap_or_else(|| { app_error!(Generic, "Failed to access mutable vault") });

        let mut vault_without_key = vault.clone();
        vault_without_key.key = None;

        let encoded_vault = ZeroByte::from_vec(
            encode_to_vec(&vault_without_key, standard())
                .map_err(|_| app_error!(Generic, "Failed to encode vault to vector"))?
        );

        let vault_location = PathBuf::from(window.get_vault_location().to_string());
        let key = vault.key.as_ref()
            .unwrap_or_else(|| { app_error!(Generic, "Failed to access vault key") });

        if let Err(e) = file::write_encrypted_file(&encoded_vault, &vault_location, key) {
            let message = 
                if cfg!(debug_assertions) { e.as_str() }
                else { "Failed to save vault." };

            file::show_dialog(Some("Error"), Some(message), Some(MessageButtons::Ok));
        }

        Ok(())
    }

    /// Saves changes to an edited vault item and refreshes display
    fn save_selected_item(window: &Weak<MainWindow>, new_item: VaultItem) -> Result<(), AppError> {
        {
            let mut vault_guard = GLOBAL_VAULT.lock().unwrap();
            if let Some(vault) = &mut *vault_guard {
                if let Some(item) = vault.items.iter_mut().find(|item| item.id == new_item.id) {
                    item.name = ZeroByte::from_shared_string(new_item.name);
                    item.username = ZeroByte::from_shared_string(new_item.username);
                    item.password = ZeroByte::from_shared_string(new_item.password);
                    item.url = ZeroByte::from_shared_string(new_item.url);
                    item.notes = ZeroByte::from_shared_string(new_item.notes);
                }
            }
        }

        let window = window.upgrade().unwrap();
        Self::save_vault_state(&window.as_weak());
        Self::load_selected_item(&window.as_weak(), new_item.id);
        Self::update_vault_items(&window);
    }

    /// Loads selected item into the UI for viewing/editing
    fn load_selected_item(window: &Weak<MainWindow>, item_id: i32) {
        let window = window.upgrade().unwrap();
        let vault_guard = GLOBAL_VAULT.lock().unwrap();
        
        if let Some(vault) = &*vault_guard {
            if let Some(item) = vault.items.iter().find(|item| item.id == item_id) {
                let selected_item = VaultItem {
                    id: item.id,
                    name: item.name.to_shared_string_secure(),
                    username: item.username.to_shared_string_secure(),
                    password: item.password.to_shared_string_secure(),
                    url: item.url.to_shared_string_secure(),
                    notes: item.notes.to_shared_string_secure(),
                };

                window.set_selected_vault_item(selected_item);
            }
        }      
    }

    /// Updates the list of vault items in the UI
    fn update_vault_items(window: &MainWindow) {
        let vault_guard = GLOBAL_VAULT.lock().unwrap();

        if let Some(vault) = &*vault_guard {
            let items: Vec<MainWindowItem> = vault.items
                .iter()
                .map(|item| MainWindowItem {
                    id: item.id,
                    name: item.name.to_shared_string_secure(),
                })
                .collect();

            window.set_vault_items(ModelRc::new(VecModel::from(items)));
        }
    }

    /// Opens a file dialog for selecting an existing vault
    fn open_unlock_vault(window: &Weak<MainWindow>) {
        if let Some(path) = Self::open_existing_vault() {
            let window = window.upgrade().unwrap();
            let path = SharedString::from(path.display().to_string());

            window.set_vault_location(path);
        }
    }

    /// Attempts to open and decrypt an existing vault file
    fn unlock_vault(window: &Weak<MainWindow>, location: String, password: &ZeroByte) {
        let window = window.upgrade().unwrap();

        let path = match PathBuf::from_str(&location) {
            Ok(p) => p,
            Err(_) => {
                file::show_dialog(Some("Error"), Some("Invalid file path"), Some(MessageButtons::Ok));
                return;
            }
        };

        let key = match file::derive_file_key(&path, password) {
            Ok(k) => k,
            Err(e) => {
                file::show_dialog(
                    Some("Error"),
                    Some(format!("Failed to derive key: {}", e).as_str()),
                    Some(MessageButtons::Ok)
                );
                return;
            }
        };

        if let Ok(bytes) = read_encrypted_file(&path, &key) {
            match bytes.with_bytes(|byte_slice| decode_from_slice(byte_slice, standard())) {   //decode_from_slice(bytes.as_bytes(), standard()) {
                Ok((decoded_bytes, _bytes_read)) => {
                    if let Ok(mut vault_guard) = GLOBAL_VAULT.lock() {
                         let mut vault: Vault = decoded_bytes;
                        vault.key = Some(key);

                        *vault_guard = Some(vault);
                        window.set_vault_open(true);
                    }
                },
                Err(e) => {
                    file::show_dialog(
                        Some("Decode Error"),
                        Some(format!("Failed to decode vault data: {}", e).as_str()),
                        Some(MessageButtons::Ok),
                    );
                    return;
                }
            }
        } else {
            file::show_dialog(Some("Error"), Some("Failed to open vault file"), Some(MessageButtons::Ok));
        }

        Self::update_vault_items(&window);
    }

    /// Opens a system file picker to select a vault file
    fn open_existing_vault() -> Option<PathBuf> {
        file::show_file_dialog(Some("Select Vault File"), Some(("Vault Files", &["vault"])), None, true)
    }

    /// Opens the CreateVaultWindow if it's not already visible and
    /// disables input on the main window while open.
    fn open_create_vault_window(_window_weak: &Weak<MainWindow>, create_vault_window_handler: &Arc<Mutex<CreateVaultWindowHandler>>) {
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
        self.visible.load(Ordering::Relaxed)
    }

    fn get_visible_arc(&self) -> Arc<AtomicBool> {
        self.visible.clone()
    }

    fn set_visible(&self, value: bool) {
        self.visible.store(value, Ordering::Relaxed);
    }

    fn initialize(&mut self) {
        if let Some(window) = self.get_window().upgrade() {
            window.window().on_close_requested( || {
                Self::cleanup();
            
                // Exit the entire program if main window is closed
                std::process::exit(0);
            });
        }
    }

    fn cleanup() {
        // Zeroize vault and clean up properly before exit
        if let Ok(mut vault_guard) = GLOBAL_VAULT.lock() {
            if let Some(mut vault) = vault_guard.take() {
                vault.zeroize();
                return;
            }

            #[cfg(debug_assertions)]
            println!("Warning: No vault found to zeroize");
            return;
        }

        #[cfg(debug_assertions)]
        eprintln!("Error: Failed to acquire vault lock for cleanup");
    }
}