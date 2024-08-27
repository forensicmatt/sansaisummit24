use std::vec;
use evtx_clustering::embedding::EmbeddingsHandler;
use openai_api_rs::v1::common::TEXT_EMBEDDING_3_SMALL;
use ndarray::Array2;
use linfa::prelude::*;
use linfa_clustering::Dbscan;

#[tokio::test]
async fn test_embeddings() {
    let openai_key = std::env::var("OPENAI_KEY").expect("OPENAI_KEY not set.");

    let embeddings = EmbeddingsHandler::new(openai_key, TEXT_EMBEDDING_3_SMALL, Some(10), 10)
        .with_cache("./cache.3small.10")
        .expect("Error adding cache.");
    let result = embeddings.get_embedding("I am a test").await;
    println!("{result:?}");
}


#[tokio::test]
async fn test_embeddings_multiple() {
    let openai_key = std::env::var("OPENAI_KEY").expect("OPENAI_KEY not set.");
    let embeddings = EmbeddingsHandler::new(openai_key, TEXT_EMBEDDING_3_SMALL, Some(10), 10)
        .with_cache("./cache.3small.10")
        .expect("Error adding cache.");

    let result = embeddings.get_embeddings(vec![
        "I am a test",
        "test test test",
        "mr test",
        "yikes...",
        "bubble bubble"
    ]).await;
    println!("{result:?}");
}


#[tokio::test]
async fn test_embeddings_multiple_cluster() {
    let openai_key = std::env::var("OPENAI_KEY").expect("OPENAI_KEY not set.");
    let embeddings = EmbeddingsHandler::new(openai_key, TEXT_EMBEDDING_3_SMALL, Some(10), 10)
        .with_cache("./cache.3small.10")
        .expect("Error adding cache.");

    let result = embeddings.get_embeddings(vec![
        "I am a test",
        "test test test",
        "mr test",
        "yikes...",
        "bubble bubble"
    ]).await.unwrap();
    
    // Its recommended to flatten a vector to turn it into an array
    let flat_vec: Vec<f32> = result
        .iter()
        .map(|v| {
            v.response.data.first().unwrap().embedding.clone()
        })
        .flatten()
        .collect();
    
    println!("{flat_vec:?}");
    
    // Project a flat vec into an array of arrays
    let dataset = Array2::<f32>::from_shape_vec(
        (5, 10),
        flat_vec
    ).unwrap();

    println!("{dataset:?}");

    let clusters = Dbscan::params(2)
        .tolerance(0.98)
        .transform(&dataset)
        .unwrap();

    println!("clusters.to_vec(); => {:?}", clusters.to_vec());

}
