use aes_gcm::{
    aead::{rand_core::RngCore, Aead, AeadCore, KeyInit, OsRng as AesOsRng}, Aes256Gcm, Key as AesKey, Error as AesError, Nonce
};
use argon2::{password_hash::Salt, Argon2, Params, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{self, SaltString, rand_core::OsRng as ArgonOsRng};


pub(super) struct ArgonKey {
    pub(super) bytes: [u8; 32],
    pub(super) salt: [u8; 16],
}

pub(super) struct Crypto {}

impl Crypto {
    pub(super) fn derive_argon_key(bytes: &[u8], salt: Option<[u8; 16]>) -> Result<ArgonKey, String> {
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
