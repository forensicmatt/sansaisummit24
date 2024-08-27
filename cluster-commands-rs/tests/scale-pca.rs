use ndarray::{array, Array2, Axis};
use linfa::traits::Fit;
use linfa::prelude::Transformer;
use linfa_preprocessing::linear_scaling::LinearScaler;
use linfa_clustering::Dbscan;


#[test]
fn test_scaler() {
    let dataset: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>> = array![
        [-0.091962464, -0.10718001, -0.6467201, 0.11911137, -0.24226725, -0.58969223, -0.13913178, -0.0023335034, -0.3460094, 0.04588012],
        [-0.791962464, -0.10748001, -0.6467201, 0.11011137, -0.14226725, -0.58969223, -0.13813178, -0.0023335034, -0.3460094, 0.04583012],
        [-0.091962464, -0.10718001, -0.6467201, 0.11911137, -0.24226725, -0.58969223, -0.13913178, -0.0023335034, -0.3460094, 0.04588012],
        [-0.091932463, -0.10718001, -0.6467201, 0.11911227, -0.24326725, -0.58969223, -0.13913178, -0.0023335034, -0.3460094, 0.04588012]
    ];

    // let scaler = LinearScaler::min_max_range(0.000, 1.0000).fit(&dataset.into()).expect("Error scaling data.");
    // println!("{:?}", scaler);

    // let transformed = scaler.transform(dataset);
    // println!("{:?}", transformed);

    let clusters = Dbscan::params(2)
        .tolerance(0.6)
        .transform(&dataset)
        .unwrap();

    println!("{:#?}", clusters);
}