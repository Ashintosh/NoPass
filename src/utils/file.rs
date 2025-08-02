use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use super::crypto::Crypto;

pub(crate) fn write_encrypted_file(bytes: &Vec<u8>, path: &PathBuf, password: String) -> Result<(), String> {
    let key = Crypto::derive_argon_key(password.as_bytes(), None)?;
    
    let encrypted_bytes = Crypto::aes_gcm_encrypt(bytes, key.bytes.to_vec())
        .map_err(|e| e.to_string())?;

    let mut combined = Vec::with_capacity(key.salt.len() + encrypted_bytes.len());
    combined.extend_from_slice(&key.salt);          // [0..16] = salt
    combined.extend_from_slice(&encrypted_bytes);   // [16..] = none + cipherbytes

    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(&combined).map_err(|e| e.to_string())?;

    Ok(())
}

pub(crate) fn read_encrypted_file(path: &PathBuf, password: String) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|e| e.to_string())?;

    if bytes.len() < 16 {
        return Err("File too short to contain valid salt".into());
    }

    let (salt_bytes, encrypted_data) = bytes.split_at(16);
    let mut salt = [0u8; 16];
    salt.copy_from_slice(salt_bytes);

    let key = Crypto::derive_argon_key(password.as_bytes(), Some(salt))?;

    let decrypted_bytes = Crypto::aes_gcm_decrypt(encrypted_data, key.bytes.to_vec())
        .map_err(|e| e.to_string())?;

    Ok(decrypted_bytes)
}