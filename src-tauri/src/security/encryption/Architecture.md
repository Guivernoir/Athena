encryption/
├── mod.rs # Public interface: encrypt, decrypt, config
├── types.rs # Key types: Key, Nonce, CipherText, etc.
├── aead/
│ ├── mod.rs # AEAD interface trait
│ ├── chacha20poly1305.rs # Rust-native AEAD impl (default)
│ └── aes_gcm.rs # Optional: AES-GCM (for compliance)
├── kdf/
│ ├── mod.rs # Trait: KeyDerivation
│ ├── argon2.rs # Argon2id secure password hashing
│ └── hkdf.rs # HKDF for deterministic key derivation
├── key.rs # Key generation, secure storage, random bytes
├── nonce.rs # Nonce generation, reuse protection
├── errors.rs # CryptoError, DecryptionError, etc.
├── utils.rs # Secure wiping, memory alignment, hex utils
└── tests/
├── mod.rs
├── test_aead.rs
├── test_kdf.rs
└── test_integration.rs
