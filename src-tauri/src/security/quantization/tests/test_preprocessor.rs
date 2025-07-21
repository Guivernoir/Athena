use quantization::preprocessor::{Preprocessor, L2Norm, MeanCenter};

#[test]
fn l2_norm_unit_vector() {
    let v = Vector::<4>::new([1.0, 0.0, 0.0, 0.0]);
    let normed = L2Norm.apply(&v);
    assert!((normed.as_slice()[0] - 1.0).abs() < 1e-6);
}

#[test]
fn mean_center_zeros() {
    let v = Vector::<4>::new([1.0, 2.0, 3.0, 4.0]);
    let centered = MeanCenter { enabled: true }.apply(&v);
    let sum: f32 = centered.as_slice().iter().sum();
    assert!(sum.abs() < 1e-6);
}