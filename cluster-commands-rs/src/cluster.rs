use serde_json::json;
use ndarray::Array2;
use linfa_clustering::Dbscan;
use linfa::prelude::Transformer;
use polars::prelude::*;
use crate::embedding::ValueEmbedding;
use crate::errors::CustomError;


pub fn get_cluster_mapping(
    value_embeddings: Vec<ValueEmbedding>,
    min_points: usize,
    tolerance: f32
) -> Result<DataFrame, CustomError>{
    // Get embeddings for command line values
    let mut _vec_values = Vec::new();
    let mut _vec_embeddings_flat: Vec<f32> = Vec::new();
    let mut _vec_embeddings_string_vec: Vec<String> = Vec::new();
    let mut dimentions: Option<usize> = None;

    value_embeddings.iter()
        .for_each(|value_embedding| {
            _vec_values.push(value_embedding.value.clone());
            // Get the embeddings from the response
            let embedding = value_embedding.response.data
                .first()
                .expect("No embedding data!")
                .embedding
                .clone();

            // Set dimentions used
            if None == dimentions {
                dimentions = Some(embedding.len());
            }

            _vec_embeddings_flat.extend(&embedding);
            _vec_embeddings_string_vec.push(format!("{}", json!(&embedding)));
        });
    
    let dimentions = dimentions.unwrap();
    let dataset = Array2::<f32>::from_shape_vec(
        (_vec_values.len(), dimentions),
        _vec_embeddings_flat
    ).unwrap();

    let clusters = Dbscan::params(min_points)
        .tolerance(tolerance)
        .transform(&dataset)
        .unwrap();

    // Map to i64 where -1 is no cluster
    let clusters: Vec::<i64> = clusters.iter()
        .map(|v|v.map_or(-1, |v|v as i64))
        .collect();

    let s1 = Series::new("value", &_vec_values);
    let s2 = Series::new("cluster", clusters);
    let s3 = Series::new("embedding", _vec_embeddings_string_vec);
    let df = DataFrame::new(vec![s1, s2, s3])?;

    Ok(df)
}