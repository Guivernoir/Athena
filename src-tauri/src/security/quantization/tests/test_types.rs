use quantization::types::{Vector, QuantizedVector, ValidationError};

#[test]
fn vector_dimension_check() {
    let v = Vector::<384>::new([0.0; 384]);
    assert_eq!(v.as_slice().len(), 384);
}

#[test]
fn quantized_vector_zero_copy() {
    let q = QuantizedVector::<48>::new([0u8; 48]);
    assert_eq!(q.as_bytes(), &[0u8; 48]);
}

#[test]
#[should_panic(expected = "DimMismatch")]
fn invalid_dimension_runtime() {
    let _ = Vector::<1025>::new([0.0; 1025]); // > MAX_DIM
}