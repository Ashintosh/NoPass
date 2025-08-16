use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use bincode::config::standard;
use bincode::serde::encode_to_vec;
use rfd::MessageButtons;
use slint::{ComponentHandle, SharedString, Weak};

use crate::CreateVaultWindow;
use crate::handlers::WindowHandler;
use crate::models::vault::Vault;
use crate::utils::crypto::Crypto;
use crate::utils::file;
use crate::utils::zerobyte::ZeroByte;


/// Coordinates the MainWindow lifecycle and UI behavior.
/// Holds ownership to prevent premature drop and supports weak upgrade for event binding.
pub(crate) struct CreateVaultWindowHandler {
    _window_strong: CreateVaultWindow,
    window: Weak<CreateVaultWindow>,
    visible: Arc<AtomicBool>,
}

impl CreateVaultWindowHandler {
    /// Creates a new `MainWindowHandler` and sets up window behavior.
    /// Panics on window creation failure (app can't continue without it).
    pub(crate) async fn new() -> Arc<Mutex<Self>> {
        let window = CreateVaultWindow::new().expect("Failed to create new MainWindow");
        let weak = window.as_weak();
        let handler = Self {
            _window_strong: window,
            window: weak,
            visible: Arc::new(AtomicBool::new(false)),
        };

        let handler = Arc::new(Mutex::new(handler));
        Self::setup(&handler).await;
        
        handler
    }

    async fn setup(handler_arc: &Arc<Mutex<Self>>) {
        let handler_arc_clone = Arc::clone(handler_arc);
        let window = handler_arc_clone.lock().unwrap().get_window().upgrade().unwrap();
        //let window_weak = window.as_weak();

        let handler_arc_clone_done = Arc::clone(handler_arc);
        window.on_create_database_done(move |password: SharedString| {
            let password = ZeroByte::from_shared_string(password);
            if let Some(vault_path) = file::show_file_dialog(
                Some("Select Vault Location"), 
                Some(("Vault Files", &["vault"])), 
                Some("passwords.vault"),
                false
            ) {
                let handler_arc_for_task = Arc::clone(&handler_arc_clone_done);
                slint::spawn_local(async move {
                    Self::create_vault_file(&vault_path, &password).await;
                    
                    if let Ok(mut handler) = handler_arc_for_task.lock() {
                        handler.hide();
                    }
                }).ok();
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
    async fn create_vault_file(path: &PathBuf, password: &ZeroByte) {
        let vault = Vault::new();

        let encoded_vault = match encode_to_vec(&vault, standard()) {
            Ok(data) => ZeroByte::from_vec(data),
            Err(e) => {
                file::show_dialog(
                    Some("Error"),
                    Some(format!("Failed to encode vault: {}", e).as_str()),
                    Some(MessageButtons::Ok)
                );
                return;
            }
        };

        let key = match Crypto::derive_argon_key(password, None) {
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

        let path_clone = path.clone();
        let result = match tokio::task::spawn_blocking(move || {
            file::write_encrypted_file(&encoded_vault, &path_clone, &key)
        }).await {
            Ok(res) => res,
            Err(e) => {
                file::show_dialog(
                    Some("Error"),
                    Some(format!("Task failed: {}", e).as_str()),
                    Some(MessageButtons::Ok)
                );
                return;
            }
        };

        match result {
            Ok(()) => file::show_dialog(
                Some("Vault Created"),
                Some(format!("Vault has been saved at {}", path.display()).as_str()),
                Some(MessageButtons::Ok),
            ),
            Err(e) => file::show_dialog(
                Some("Error"),
                if cfg!(debug_assertions) { Some(e.as_str()) }
                    else { Some("Failed to create vault file.") },
                Some(MessageButtons::Ok),
            )
        };
    }
}

impl WindowHandler for CreateVaultWindowHandler {
    type Component = CreateVaultWindow;

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