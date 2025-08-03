use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;

use crate::utils::crypto::ArgonKey;

use super::crypto::Crypto;


pub(crate) fn derive_file_key(path: &PathBuf, password: &String) -> Result<ArgonKey, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;

    let mut salt = [0u8; 16];
    file.read_exact(&mut salt).map_err(|e| e.to_string())?;

    Ok(Crypto::derive_argon_key(password.as_bytes(), Some(salt))?)
}

pub(crate) fn write_encrypted_file(bytes: &Vec<u8>, path: &PathBuf, key: &ArgonKey) -> Result<(), String> {    
    let encrypted_bytes = Crypto::aes_gcm_encrypt(bytes, key.bytes.to_vec())
        .map_err(|e| e.to_string())?;

    let mut combined = Vec::with_capacity(key.salt.len() + encrypted_bytes.len());
    combined.extend_from_slice(&key.salt);          // [0..16] = salt
    combined.extend_from_slice(&encrypted_bytes);   // [16..] = none + cipherbytes

    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(&combined).map_err(|e| e.to_string())?;

    Ok(())
}

pub(crate) fn read_encrypted_file(path: &PathBuf, key: &ArgonKey) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);

    let mut salt = [0u8; 16];
    reader.read_exact(&mut salt).map_err(|e| e.to_string())?;

    let mut encrypted_data = Vec::new();
    reader.read_to_end(&mut encrypted_data).map_err(|e| e.to_string())?;

    let decrypted_bytes = Crypto::aes_gcm_decrypt(&encrypted_data, key.bytes.to_vec())
        .map_err(|e| e.to_string())?;

    Ok(decrypted_bytes)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    const TEST_BYTES: &[u8] = b"Super secret message";
    const TEST_PASSWORD: &str = "correct-horse-battery-staple";

    #[test]
    fn test_write_encrypted_file_create_files() {
        let bytes = TEST_BYTES.to_vec();
        let password = TEST_PASSWORD.to_string();
        let key = Crypto::derive_argon_key(password.as_bytes(), None).expect("Key derivation failed");

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
        let bytes = TEST_BYTES.to_vec();
        let password = TEST_PASSWORD.to_string();

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_path_buf();
        let key = Crypto::derive_argon_key(password.as_bytes(), None).expect("Key derivation failed");
        write_encrypted_file(&bytes, &path, &key).expect("Write failed");

        // TODO: Test if returned key is the same
        let decrypted = read_encrypted_file(&path, &key).expect("Read failed");
        assert_eq!(decrypted, bytes);
    }

    #[test]
    fn test_decrypt_with_wrong_password_fails() {
        let bytes = TEST_BYTES.to_vec();
        
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_path_buf();

        let correct_password = TEST_PASSWORD.to_string();
        let correct_key = Crypto::derive_argon_key(correct_password.as_bytes(), None).expect("Key derivation failed");
        write_encrypted_file(&bytes, &path, &correct_key).expect("Write failed");

        let wrong_password = "incorrect".to_string();
        let wrong_key = Crypto::derive_argon_key(wrong_password.as_bytes(), None).expect("Key derivation failed");
        let result = read_encrypted_file(&path, &wrong_key);
        assert!(result.is_err(), "Decryption should fail with wrong password");
    }

    #[test]
    fn test_invalid_file_format_fails() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_path_buf();
        // Write invalid content (too short for salt)
        fs::write(&path, b"short").expect("Failed to write");

        let wrong_key = Crypto::derive_argon_key(b"incorrect", None).expect("Key derivation failed");
        let result = read_encrypted_file(&path, &wrong_key);
        assert!(result.is_err(), "Should fail on invalid input");
    }
}