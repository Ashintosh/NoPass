use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;

use rfd::{MessageButtons, MessageDialogResult};

use crate::utils::crypto::ArgonKey;
use crate::utils::zerobyte::ZeroByte;

use super::crypto::Crypto;


pub(crate) fn derive_file_key(path: &PathBuf, password: &ZeroByte) -> Result<ArgonKey, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;

    let mut salt = [0u8; 16];
    file.read_exact(&mut salt).map_err(|e| e.to_string())?;

    Ok(Crypto::derive_argon_key(password, Some(salt))?)
}

pub(crate) fn write_encrypted_file(bytes: &ZeroByte, path: &PathBuf, key: &ArgonKey) -> Result<(), String> {    
    let encrypted_bytes = Crypto::aes_gcm_encrypt(bytes, key)
        .map_err(|e| e.to_string())?;

    let mut combined = Vec::with_capacity(key.salt.len() + encrypted_bytes.len());
    combined.extend_from_slice(&key.salt);                     // [0..16] = salt

    encrypted_bytes.with_bytes(|bytes_slice| {
        combined.extend_from_slice(bytes_slice);              // [16..] = none + cipherbytes
    });

    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(&combined).map_err(|e| e.to_string())?;

    Ok(())
}

pub(crate) fn read_encrypted_file(path: &PathBuf, key: &ArgonKey) -> Result<ZeroByte, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);

    let mut salt = [0u8; 16];
    reader.read_exact(&mut salt).map_err(|e| e.to_string())?;

    let mut encrypted_data = Vec::new();
    reader.read_to_end(&mut encrypted_data).map_err(|e| e.to_string())?;

    let decrypted_bytes = Crypto::aes_gcm_decrypt(&ZeroByte::from_vec(encrypted_data), key)
        .map_err(|e| e.to_string())?;

    Ok(decrypted_bytes)
}

/// Opens a save file dialog and returns the user-selected path (if any).
pub(crate) fn show_file_dialog(title: Option<&str>, filters: Option<(&str, &[&str])>, file_name: Option<&str>, pick: bool) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new();

    if let Some(title) = title {
        dialog = dialog.set_title(title);
    }

    if let Some(filters) = filters {
        dialog = dialog.add_filter(filters.0, filters.1);
    }

    if let Some(file_name) = file_name {
        dialog = dialog.set_file_name(file_name);
    }

    let handle = std::thread::spawn(move || {
        match pick {
            true => dialog.pick_file(),
            false => dialog.save_file()
        }
    });

    handle.join().ok()?
}

pub(crate) fn show_dialog(title: Option<&str>, message: Option<&str>, buttons: Option<MessageButtons>) -> Option<MessageDialogResult> {
    let mut dialog = rfd::MessageDialog::new();

    if let Some(title) = title {
        dialog = dialog.set_title(title);
    }

    if let Some(message) = message {
        dialog = dialog.set_description(message);
    }

    if let Some(buttons) = buttons {
        dialog = dialog.set_buttons(buttons);
    }

    let handle = std::thread::spawn(|| {
        dialog.show()
    });

    handle.join().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    const TEST_BYTES: &[u8] = b"Super secret message";
    const TEST_PASSWORD: &[u8] = b"correct-horse-battery-staple";

    #[test]
    fn test_write_encrypted_file_create_files() {
        let bytes = ZeroByte::from_bytes(TEST_BYTES);
        let password = ZeroByte::from_bytes(TEST_PASSWORD);
        let key = Crypto::derive_argon_key(&password, None).expect("Key derivation failed");

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_path_buf();
        write_encrypted_file(&bytes, &path, &key).expect("Write failed");

        // Assert the file exists and has some size
        let metadata = std::fs::metadata(&path).expect("File not found");
        assert!(metadata.len() > 16, "File too small to contain salt and data");

        // Check the first 16 bytes for salt
        let mut contents = Vec::new();
        std::fs::File::open(&path).unwrap().read_to_end(&mut contents).unwrap();
        assert_eq!(contents.len() as u64, metadata.len());

        // Salt is first 16 bytes
        assert_eq!(contents[..16].len(), 16);
    }

    #[test]
    fn test_read_encrypted_file_returns_data() {
        let bytes = ZeroByte::from_bytes(TEST_BYTES);
        let password = ZeroByte::from_bytes(TEST_PASSWORD);

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_path_buf();
        let key = Crypto::derive_argon_key(&password, None).expect("Key derivation failed");
        write_encrypted_file(&bytes, &path, &key).expect("Write failed");

        // TODO: Test if returned key is the same
        let decrypted = read_encrypted_file(&path, &key).expect("Read failed");

        decrypted.with_bytes(|decrypted_slice| {
            bytes.with_bytes(|bytes_slice| {
                assert_eq!(decrypted_slice, bytes_slice);
            });
        });
    }

    #[test]
    fn test_decrypt_with_wrong_password_fails() {
        let bytes = ZeroByte::from_bytes(TEST_BYTES);
        
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_path_buf();

        let correct_password = ZeroByte::from_bytes(TEST_PASSWORD);
        let correct_key = Crypto::derive_argon_key(&correct_password, None).expect("Key derivation failed");
        write_encrypted_file(&bytes, &path, &correct_key).expect("Write failed");

        let wrong_password = ZeroByte::from_bytes(b"incorrect");
        let wrong_key = Crypto::derive_argon_key(&wrong_password, None).expect("Key derivation failed");
        let result = read_encrypted_file(&path, &wrong_key);
        assert!(result.is_err(), "Decryption should fail with wrong password");
    }

    #[test]
    fn test_invalid_file_format_fails() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_path_buf();
        // Write invalid content (too short for salt)
        fs::write(&path, b"short").expect("Failed to write");

        let wrong_key = Crypto::derive_argon_key(&ZeroByte::from_bytes(b"incorrect"), None).expect("Key derivation failed");
        let result = read_encrypted_file(&path, &wrong_key);
        assert!(result.is_err(), "Should fail on invalid input");
    }
}