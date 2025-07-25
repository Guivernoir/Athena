#[cfg(test)]
mod tests {
    use super::*;
    use crate::kdf::KeyDerivation as _;

    #[test]
    fn derive_basic() {
        let ikm = SecretKey::random(32).unwrap();
        let k1 = HkdfSha256::derive_key(&ikm, b"ctx1", 32).unwrap();
        let k2 = HkdfSha256::derive_key(&ikm, b"ctx2", 32).unwrap();
        assert_ne!(k1.as_bytes(), k2.as_bytes());
    }

    #[test]
    fn boundary_lengths() {
        let ikm = SecretKey::random(32).unwrap();
        assert!(HkdfSha256::derive_key(&ikm, b"", 8160).is_ok());
        assert!(HkdfSha256::derive_key(&ikm, b"", 8161).is_err());
    }
}