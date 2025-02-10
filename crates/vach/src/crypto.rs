#![cfg(feature = "crypto")]
#![cfg_attr(docsrs, doc(cfg(feature = "crypto")))]
use std::fmt;

use aes_gcm::aead::Aead;
use aes_gcm::aes::cipher::consts::U12;
use aes_gcm::{Aes256Gcm, Nonce, KeyInit};

pub use ed25519_dalek::{SigningKey, VerifyingKey, Signature};

use crate::prelude::{InternalResult, InternalError};

/// Encryption - Decryption, A convenient wrapper around [`aes`](aes_gcm) encryption and decryption
pub(crate) struct Encryptor {
	cipher: Aes256Gcm,
	nonce: Nonce<U12>,
}

impl fmt::Debug for Encryptor {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "[Vach::Encryptor] cipher: Aes256Gcm, nonce: {:?}", self.nonce)
	}
}

impl Encryptor {
	pub(crate) fn new(vk: &VerifyingKey) -> Encryptor {
		// Build encryption key
		let bytes = &vk.to_bytes();

		// Build Nonce
		let mut v = [178, 5, 239, 228, 165, 44, 169, 0, 0, 0, 0, 0];
		v[7..12].copy_from_slice(&crate::MAGIC);

		Encryptor {
			cipher: Aes256Gcm::new_from_slice(bytes).unwrap(),
			nonce: *Nonce::from_slice(v.as_slice()),
		}
	}

	// The meat and the mass of this struct
	pub(crate) fn encrypt(&self, data: &[u8]) -> InternalResult<Vec<u8>> {
		self.cipher
			.encrypt(&self.nonce, data)
			.map_err(InternalError::CryptoError)
	}

	pub(crate) fn decrypt(&self, data: &[u8]) -> InternalResult<Vec<u8>> {
		self.cipher
			.decrypt(&self.nonce, data)
			.map_err(InternalError::CryptoError)
	}
}
