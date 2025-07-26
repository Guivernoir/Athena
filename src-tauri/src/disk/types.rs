use core::fmt::{Debug, Display, Formatter, Result as FmtResult};
use core::cmp::{PartialEq, Eq, PartialOrd, Ord, Ordering};
use core::hash::Hash;
use alloc::vec::Vec;
use bitflags::bitflags;

/// Unique identifier for a vector in the database.
/// 
/// # Invariants
/// - Must be globally unique within a collection
/// - The underlying u64 is never 0 (reserved for sentinel values)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct VectorId(pub u64);

impl VectorId {
    /// Creates a new VectorId after validating the input.
    /// 
    /// # Panics
    /// - If `id == 0` (violates sentinel value invariant)
    pub fn new(id: u64) -> Self {
        assert!(id != 0, "VectorId cannot be 0 (reserved for sentinel values)");
        VectorId(id)
    }

    /// Returns the raw u64 value of the VectorId.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Result of a similarity search, ordered by score (descending).
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Neighbor {
    pub id: VectorId,
    pub score: f32,
}

impl Eq for Neighbor {}

impl PartialOrd for Neighbor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Ord for Neighbor {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl PartialEq for Neighbor {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Neighbor {
    /// Creates a new Neighbor with the given VectorId and similarity score.
    pub fn new(id: VectorId, score: f32) -> Self {
        Neighbor { id, score }
    }
}

/// Identifier for an IVF partition centroid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CentroidId(pub u16);

impl CentroidId {
    /// Creates a new CentroidId.
    pub fn new(id: u16) -> Self {
        CentroidId(id)
    }

    /// Returns the raw u16 value of the CentroidId.
    pub fn as_u16(&self) -> u16 {
        self.0
    }
}

/// Encrypted, packed, and compressed vector payload.
/// 
/// # Safety
/// - The contents are opaque and should only be decrypted by the appropriate crypto module
/// - This type is Send + Sync because the encrypted bytes can be safely moved between threads
#[derive(Debug, Clone)]
pub struct EncryptedBlob {
    pub bytes: Vec<u8>,
}

impl Drop for EncryptedBlob {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.bytes.zeroize();
    }
}

impl EncryptedBlob {
    /// Creates a new EncryptedBlob from raw bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        EncryptedBlob { bytes }
    }

    /// Returns the length of the encrypted blob in bytes.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns true if the encrypted blob is empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

// SAFETY: EncryptedBlob is Send + Sync because Vec<u8> is Send + Sync
// and we don't perform any interior mutability.
unsafe impl Send for EncryptedBlob {}
unsafe impl Sync for EncryptedBlob {}

/// Links a stored vector to its IVF partition and storage location.
/// 
/// # Serialization
/// This type is persisted to disk and must maintain backward compatibility.
/// The binary layout is:
/// - VectorId (8 bytes)
/// - CentroidId (2 bytes)
/// - Offset (8 bytes)
#[derive(Debug, Copy)]
pub struct IndexEntry {
    pub id: VectorId,
    pub centroid: CentroidId,
    pub offset: u64,
}

impl IndexEntry {
    /// Creates a new IndexEntry with the given parameters.
    pub fn new(id: VectorId, centroid: CentroidId, offset: u64) -> Self {
        IndexEntry { id, centroid, offset }
    }
}

/// Configuration for a vector search operation.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub top_k: usize,
    pub metric: SimilarityMetric,
}

impl SearchConfig {
    /// Creates a new SearchConfig with validation.
    /// 
    /// # Panics
    /// - If `top_k == 0` (must request at least one result)
    pub fn new(top_k: usize, metric: SimilarityMetric) -> Self {
        assert!(top_k > 0, "top_k must be greater than 0");
        SearchConfig { top_k, metric }
    }
}

/// Distance metric used for vector similarity calculations.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum SimilarityMetric {
    Cosine,
    Dot,
    L2,
}

impl SimilarityMetric {
    /// Returns the string representation of the metric.
    pub fn as_str(&self) -> &'static str {
        match self {
            SimilarityMetric::Cosine => "cosine",
            SimilarityMetric::Dot => "dot",
            SimilarityMetric::L2 => "l2",
        }
    }
}

/// Error type for disk operations (only available with `std` feature).
#[cfg(feature = "std")]
#[derive(Debug)]
#[non_exhaustive]
pub enum DiskError {
    Io(std::io::Error),
    InvalidVector,
    NotFound,
    CorruptIndex,
    InvalidOffset,
    DecryptionError,
}

#[cfg(feature = "std")]
impl From<std::io::Error> for DiskError {
    fn from(err: std::io::Error) -> Self {
        DiskError::Io(err)
    }
}

#[cfg(feature = "std")]
impl Display for DiskError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            DiskError::Io(e) => write!(f, "IO error: {}", e),
            DiskError::InvalidVector => write!(f, "Invalid vector data"),
            DiskError::NotFound => write!(f, "Resource not found"),
            DiskError::CorruptIndex => write!(f, "Corrupt index data"),
            DiskError::InvalidOffset => write!(f, "Invalid offset"),
            DiskError::DecryptionError => write!(f, "Decryption failed"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DiskError {}

bitflags::bitflags! {
    /// Flags controlling vector insertion behavior.
    pub struct InsertFlags: u8 {
        /// No special behavior
        const NONE = 0;
        /// Vector cannot be modified after insertion
        const IMMUTABLE = 0b00000001;
        /// Vector should be optimized for frequent access
        const HOT_PATH = 0b00000010;
    }
}

impl InsertFlags {
    /// Returns true if the IMMUTABLE flag is set.
    pub fn is_immutable(&self) -> bool {
        self.contains(InsertFlags::IMMUTABLE)
    }

    /// Returns true if the HOT_PATH flag is set.
    pub fn is_hot_path(&self) -> bool {
        self.contains(InsertFlags::HOT_PATH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cmp::Ordering;

    #[test]
    fn test_vector_id_validation() {
        assert_eq!(VectorId::new(1).as_u64(), 1);
    }

    #[test]
    #[should_panic(expected = "VectorId cannot be 0")]
    fn test_vector_id_zero() {
        VectorId::new(0);
    }

    #[test]
    fn test_neighbor_ordering() {
        let a = Neighbor::new(VectorId::new(1), 0.8);
        let b = Neighbor::new(VectorId::new(2), 0.9);
        assert_eq!(a.cmp(&b), Ordering::Less);
        assert_eq!(b.cmp(&a), Ordering::Greater);
    }

    #[test]
    fn test_search_config_validation() {
        assert_eq!(SearchConfig::new(1, SimilarityMetric::Cosine).top_k, 1);
    }

    #[test]
    #[should_panic(expected = "top_k must be greater than 0")]
    fn test_search_config_zero_topk() {
        SearchConfig::new(0, SimilarityMetric::Cosine);
    }

    #[test]
    fn test_insert_flags() {
        let mut flags = InsertFlags::NONE;
        assert!(!flags.is_immutable());
        
        flags |= InsertFlags::IMMUTABLE;
        assert!(flags.is_immutable());
    }

    // Compile-time tests for thread safety
    const _: () = {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        assert_send::<EncryptedBlob>();
        assert_sync::<EncryptedBlob>();
        assert_send::<Neighbor>();
        assert_send::<IndexEntry>();
    };
}

// Serde support (optional feature)
#[cfg(feature = "serde")]
mod serialization {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "VectorId")]
    struct VectorIdDef(u64);

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "CentroidId")]
    struct CentroidIdDef(u16);

    #[derive(Serialize, Deserialize)]
    pub struct IndexEntryDef {
        #[serde(with = "VectorIdDef")]
        id: VectorId,
        #[serde(with = "CentroidIdDef")]
        centroid: CentroidId,
        offset: u64,
    }

    impl From<IndexEntryDef> for IndexEntry {
        fn from(def: IndexEntryDef) -> Self {
            IndexEntry::new(def.id, def.centroid, def.offset)
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for IndexEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serialization::IndexEntryDef;
        IndexEntryDef {
            id: self.id,
            centroid: self.centroid,
            offset: self.offset,
        }.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for IndexEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serialization::IndexEntryDef;
        IndexEntryDef::deserialize(deserializer).map(Into::into)
    }
}