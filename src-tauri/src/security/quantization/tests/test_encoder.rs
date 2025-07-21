use quantization::{Codebook, PqEncoder};

#[test]
fn encode_decode_round_trip() {
    let centroids = [[1.0; 8]; 256]; // D=384, M=48 → 8-D sub-vectors
    let codebook = Codebook::<384, 48>::new(centroids).unwrap();
    let encoder = PqEncoder::<384, 48>::new(codebook);
    let v = Vector::<384>::new([0.5; 384]);
    let q = encoder.encode(&v);
    assert_eq!(q.as_bytes().len(), 48);
}