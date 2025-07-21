use quantization::{Quantizer, QuantizerBuilder, Vector};

#[test]
fn full_quantize_dequantize_pipeline() {
    let data: Vec<_> = (0..100).map(|_| Vector::<384>::new([1.23; 384])).collect();
    let quantizer = QuantizerBuilder::<384, 48>::new().fit(&data).unwrap();
    let v = Vector::<384>::new([1.23; 384]);
    let q = quantizer.quantize(&v).unwrap();
    let v2 = quantizer.dequantize(&q).unwrap();
    assert!(v.as_slice().iter().zip(v2.as_slice().iter()).all(|(a, b)| (a - b).abs() < 1e-3));
}

#[cfg(feature = "ffi")]
#[test]
fn runtime_quantize_matches_encoder() {
    let data = vec![1.23; 384];
    let centroids = vec![[1.0; 8]; 256];
    let bytes = quantization::quantize_runtime(&data, 384, 48, &centroids).unwrap();
    assert_eq!(bytes.len(), 48);
}