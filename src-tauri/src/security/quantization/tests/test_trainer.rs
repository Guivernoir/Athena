use quantization::trainer::{KMeans, InitMethod};

#[test]
fn kmeans_basic_clustering() {
    let data = vec![Vector::<4>::new([0.0, 0.0, 0.0, 0.0]); 100];
    let trainer = KMeans::<4, 2>::default();
    let codebook = trainer.fit(&data).unwrap();
    assert_eq!(codebook.centroids.len(), 2);
}