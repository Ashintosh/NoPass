use aes_gcm::{
    aead::{rand_core::RngCore, Aead, AeadCore, KeyInit, OsRng as AesOsRng}, Aes256Gcm, Key as AesKey, Error as AesError, Nonce
};
use argon2::{password_hash::Salt, Argon2, Params, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{self, SaltString, rand_core::OsRng as ArgonOsRng};
use serde::{Serialize, Deserialize};


#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct ArgonKey {
    pub(super) bytes: [u8; 32],
    pub(super) salt: [u8; 16],
}

pub(crate) struct Crypto {}

impl Crypto {
    pub(crate) fn derive_argon_key(bytes: &[u8], salt: Option<[u8; 16]>) -> Result<ArgonKey, String> {
        let memory = 15000;   // 15 MB
        let transform = 50;   // 50 rounds
        let parallel = 2;     // 2 threads

        let params = 
            Params::new(memory, transform, parallel, None)
                .map_err(|e| e.to_string())?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    
        let mut salt_bytes = [0u8; 16];  // 128-bit salt
        if let Some(salt) = salt {
            salt_bytes = salt;
        } else {
            ArgonOsRng.fill_bytes(&mut salt_bytes);
        }

        let mut key = [0u8; 32];
        argon2
            .hash_password_into(bytes, &salt_bytes, &mut key)
            .map_err(|e| e.to_string())?;

        Ok(ArgonKey {
            bytes: key,
            salt: salt_bytes,
        })
    }

    pub(super) fn aes_gcm_encrypt(bytes: &[u8], key: Vec<u8>) -> Result<Vec<u8>, AesError> {
        let key = AesKey::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Aes256Gcm::generate_nonce(&mut AesOsRng);

        let cipherbytes = cipher.encrypt(&nonce, bytes)?;

        let mut encrypted_bytes = nonce.to_vec();
        encrypted_bytes.extend_from_slice(&cipherbytes);

        Ok(encrypted_bytes)
    }

    pub(super) fn aes_gcm_decrypt(bytes: &[u8], key: Vec<u8>) -> Result<Vec<u8>, AesError> {
        let key = AesKey::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(&key);

        let (nonce_bytes, cipherbytes) = bytes.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let decrypted_bytes = cipher.decrypt(nonce, cipherbytes)?;
        Ok(decrypted_bytes)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const TEST_BYTES: &[u8] = b"Super secret message";
    const TEST_PASSWORD: &[u8] = b"correct-horse-battery-staple";

    #[test]
    fn test_derive_argon_key_is_deterministic_with_same_salt() {
        let salt = [42u8; 16];
        let key1 = Crypto::derive_argon_key(TEST_PASSWORD, Some(salt)).expect("Key derivation failed");
        let key2 = Crypto::derive_argon_key(TEST_PASSWORD, Some(salt)).expect("Key derivation failed");

        assert_eq!(key1.bytes, key2.bytes);
        assert_eq!(key1.salt, salt);
    }

    #[test]
    fn test_derive_argon_key_generates_unique_salts() {
        let key1 = Crypto::derive_argon_key(TEST_PASSWORD, None).expect("Key derivation failed");
        let key2 = Crypto::derive_argon_key(TEST_PASSWORD, None).expect("Key derivation failed");

        assert_ne!(key1.salt, key2.salt);
        assert_ne!(key1.bytes, key2.bytes);
    }

    #[test]
    fn test_encrypt_and_decrypt_returns_original_data() {
        let key = Crypto::derive_argon_key(TEST_PASSWORD, None).expect("Key derivation failed");
        let encrypted = Crypto::aes_gcm_encrypt(TEST_BYTES, key.bytes.to_vec()).expect("Encryption failed");

        let decrypted = Crypto::aes_gcm_decrypt(&encrypted, key.bytes.to_vec()).expect("Decryption failed");

        assert_eq!(decrypted, TEST_BYTES.to_vec());
    }

    #[test]
    fn test_encrypt_produces_different_cipherbytes_each_time() {
        let key = Crypto::derive_argon_key(TEST_PASSWORD, None).expect("Key derivation failed");

        let cipherbytes1 = Crypto::aes_gcm_encrypt(TEST_BYTES, key.bytes.to_vec()).expect("Encryption failed");
        let cipherbytes2 = Crypto::aes_gcm_encrypt(TEST_BYTES, key.bytes.to_vec()).expect("Encryption failed");

        assert_ne!(cipherbytes1, cipherbytes2, "Cipherbytes should differ due to random nonces");
    }

    #[test]
    fn test_decrypt_fails_with_wrong_key() {
        let correct_key = Crypto::derive_argon_key(TEST_PASSWORD, None).expect("Key derivation failed");
        let wrong_key = Crypto::derive_argon_key(b"incorrect", None).expect("Key derivation failed");

        let cipherbytes = Crypto::aes_gcm_encrypt(TEST_BYTES, correct_key.bytes.to_vec()).expect("Encryption failed");
        let result = Crypto::aes_gcm_decrypt(&cipherbytes, wrong_key.bytes.to_vec());

        assert!(result.is_err(), "Decryption should fail with wrong key");
    }

    #[test]
    fn test_decrypt_fails_with_tampered_cipherbytes() {
        let key = Crypto::derive_argon_key(TEST_PASSWORD, None).expect("Key derivation failed");
        let mut cipherbytes = Crypto::aes_gcm_encrypt(TEST_BYTES, key.bytes.to_vec()).expect("Encryption failed");

        // Flip a byte in the cipherbytes
        let last_index = cipherbytes.len() - 1;
        cipherbytes[last_index] ^= 0xFF;

        let result = Crypto::aes_gcm_decrypt(&cipherbytes, key.bytes.to_vec());
        assert!(result.is_err(), "Tampered cipherbytes should fail to decrypt");
    }
}