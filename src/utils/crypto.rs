use aes_gcm::{
    aead::{rand_core::RngCore, Aead, AeadCore, KeyInit, OsRng as AesOsRng},
    Aes256Gcm, Error as AesError, Key as AesKey, Nonce
};
use argon2::{Argon2, Params};
use argon2::password_hash::{rand_core::OsRng as ArgonOsRng};
use serde::{Serialize, Deserialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::utils::zerobyte::ZeroByte;

#[derive(Clone, Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop)]
pub(crate) struct ArgonKey {
    pub(super) bytes: [u8; 32],
    pub(super) salt: [u8; 16],
}

pub(crate) struct Crypto {}

impl Crypto {
    pub(crate) fn derive_argon_key(bytes: &ZeroByte, salt: Option<[u8; 16]>) -> Result<ArgonKey, String> {
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

        bytes.with_bytes(|password_bytes| {
            argon2.hash_password_into(password_bytes, &salt_bytes, &mut key)
                .map_err(|e| e.to_string())
        })?;

        Ok(ArgonKey {
            bytes: key,
            salt: salt_bytes,
        })
    }

    pub(super) fn aes_gcm_encrypt(bytes: &ZeroByte, key: &ArgonKey) -> Result<ZeroByte, AesError> {
        let key = AesKey::<Aes256Gcm>::from_slice(&key.bytes);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Aes256Gcm::generate_nonce(&mut AesOsRng);

        let cipherbytes = bytes.with_bytes(|byte_slice| {
            // TODO: Remove unwrap when adding better error handling
            ZeroByte::from_vec(cipher.encrypt(&nonce, byte_slice).unwrap())
        });

        let mut encrypted_bytes = ZeroByte::from_bytes(&nonce);
        encrypted_bytes.extend_from_zero_byte(&cipherbytes);

        Ok(encrypted_bytes)
    }

    pub(super) fn aes_gcm_decrypt(bytes: &ZeroByte, key: &ArgonKey) -> Result<ZeroByte, AesError> {
        let key = AesKey::<Aes256Gcm>::from_slice(&key.bytes);
        let cipher = Aes256Gcm::new(&key);

        let (nonce_bytes, cipherbytes) = bytes.secure_split_at(12);

        let result = nonce_bytes.with_bytes(|nonce_slice| {
            let nonce = Nonce::from_slice(nonce_slice);
            cipherbytes.with_bytes(|cipher_slice| {
                cipher.decrypt(nonce, cipher_slice)
            })
        })?;

        Ok(ZeroByte::from_vec(result))
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
        let password = ZeroByte::from_bytes(TEST_PASSWORD);
        let key1 = Crypto::derive_argon_key(&password, Some(salt)).expect("Key derivation failed");
        let key2 = Crypto::derive_argon_key(&password, Some(salt)).expect("Key derivation failed");

        assert_eq!(key1.bytes, key2.bytes);
        assert_eq!(key1.salt, salt);
    }

    #[test]
    fn test_derive_argon_key_generates_unique_salts() {
        let password = ZeroByte::from_bytes(TEST_PASSWORD);
        let key1 = Crypto::derive_argon_key(&password, None).expect("Key derivation failed");
        let key2 = Crypto::derive_argon_key(&password, None).expect("Key derivation failed");

        assert_ne!(key1.salt, key2.salt);
        assert_ne!(key1.bytes, key2.bytes);
    }

    #[test]
    fn test_encrypt_and_decrypt_returns_original_data() {
        let password = ZeroByte::from_bytes(TEST_PASSWORD);
        let key = Crypto::derive_argon_key(&password, None).expect("Key derivation failed");

        let bytes = ZeroByte::from_bytes(TEST_BYTES);
        let encrypted = Crypto::aes_gcm_encrypt(&bytes, &key).expect("Encryption failed");

        let decrypted = Crypto::aes_gcm_decrypt(&encrypted, &key).expect("Decryption failed");

        decrypted.with_bytes(|byte_slice| {
            assert_eq!(byte_slice, TEST_BYTES);
        });
    }

    #[test]
    fn test_encrypt_produces_different_cipherbytes_each_time() {
        let password = ZeroByte::from_bytes(TEST_PASSWORD);
        let key = Crypto::derive_argon_key(&password, None).expect("Key derivation failed");

        let bytes = ZeroByte::from_bytes(TEST_BYTES);
        let cipherbytes1 = Crypto::aes_gcm_encrypt(&bytes, &key).expect("Encryption failed");
        let cipherbytes2 = Crypto::aes_gcm_encrypt(&bytes, &key).expect("Encryption failed");

        cipherbytes1.with_bytes(|byte_slice1| {
            cipherbytes2.with_bytes(|byte_slice2| {
                assert_ne!(byte_slice1, byte_slice2, "Cipherbytes should differ due to random nonces");
            });
        });
    }

    #[test]
    fn test_decrypt_fails_with_wrong_key() {
        let password = ZeroByte::from_bytes(TEST_PASSWORD);
        let correct_key = Crypto::derive_argon_key(&password, None).expect("Key derivation failed");
        let wrong_key = Crypto::derive_argon_key(&ZeroByte::from_bytes(b"incorrect"), None).expect("Key derivation failed");

        let bytes = ZeroByte::from_bytes(TEST_BYTES);
        let cipherbytes = Crypto::aes_gcm_encrypt(&bytes, &correct_key).expect("Encryption failed");
        let result = Crypto::aes_gcm_decrypt(&cipherbytes, &wrong_key);

        assert!(result.is_err(), "Decryption should fail with wrong key");
    }

    #[test]
    fn test_decrypt_fails_with_tampered_cipherbytes() {
        let password = ZeroByte::from_bytes(TEST_PASSWORD);
        let key = Crypto::derive_argon_key(&password, None).expect("Key derivation failed");

        let bytes = ZeroByte::from_bytes(TEST_BYTES);
        let mut cipherbytes = Crypto::aes_gcm_encrypt(&bytes, &key).expect("Encryption failed");

        // Flip a byte in the cipherbytes
        let last_index = cipherbytes.len() - 1;
        cipherbytes.with_bytes_mut(|data| {
            data[last_index] ^= 0xFF;
        });

        let result = Crypto::aes_gcm_decrypt(&cipherbytes, &key);
        assert!(result.is_err(), "Tampered cipherbytes should fail to decrypt");
    }
}