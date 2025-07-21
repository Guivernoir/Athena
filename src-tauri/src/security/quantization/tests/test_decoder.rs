use quantization::{Codebook, PqDecoder};

#[test]
fn decode_exact_reconstruction() {
    let centroids = [[1.0; 8]; 256];
    let codebook = Codebook::<384, 48>::new(centroids).unwrap();
    let decoder = PqDecoder::<384, 48>::new(codebook);
    let q = QuantizedVector::<48>::new([0u8; 48]); // index 0 everywhere
    let v = decoder.decode(&q);
    assert!(v.as_slice().iter().all(|&x| (x - 1.0).abs() < 1e-6));
}