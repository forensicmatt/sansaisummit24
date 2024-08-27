use std::path::Path;
use std::sync::Arc;
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::embedding::{EmbeddingResponse, EmbeddingRequest};
use sled::Db;
use sled::transaction::TransactionResult;
use tokio::sync::Semaphore;
use crate::errors::CustomError;


/// Get an embedding response
async fn get_embedding(
    client: Arc<OpenAIClient>, 
    model: String, 
    dimensions: Option<i32>, 
    input: impl AsRef<str>
) -> Result<EmbeddingResponse, CustomError> {
    let mut request = EmbeddingRequest::new(
        model.clone(), 
        input.as_ref().to_string()
    );
    request.dimensions = dimensions;
    client.embedding(request).await.map_err(|e|e.into())
}


#[derive(Debug)]
pub struct ValueEmbedding{
    pub value: String, 
    pub response: EmbeddingResponse
}
impl ValueEmbedding {
    pub fn new(value: String, response: EmbeddingResponse) -> Self {
        Self {
            value, 
            response
        }
    }
}


pub struct EmbeddingsHandler {
    client: Arc<OpenAIClient>,
    model: String,
    dimensions: Option<i32>,
    parallel_requests: usize,
    cache: Option<Db>
}
impl EmbeddingsHandler {
    /// Create Embeddings struct for handling embedding operations.
    pub fn new(api_key: String, model: impl AsRef<str>, dimensions: Option<i32>, parallel_requests: usize) -> Self {
        let client = Arc::new(OpenAIClient::new(api_key));
        Self {
            client,
            model: model.as_ref().to_string(),
            dimensions,
            parallel_requests,
            cache: None
        }
    }

    pub fn with_cache(mut self, path: impl AsRef<Path>) -> Result<Self, CustomError> {
        let path = path.as_ref();
        if path.is_dir() {
            // Cache already exists so we need to check meta is applicable to embedding options.
            let db = sled::open(&path)?;
            match db.get(b"model") {
                // Validate that model is correct
                Ok(Some(value_bytes)) => {
                    if value_bytes != self.model.as_bytes() {
                        return Err(CustomError::cache_error(format!(
                            "{:?} saved model {} does not match current model {}.",
                            path,
                            String::from_utf8_lossy(&value_bytes),
                            self.model
                        )))
                    }
                }
                Ok(None) => return Err(CustomError::cache_error(format!("{:?} had no model key.", path))),
                Err(e) => return Err(e.into()),
            }

            if let Some(current_dimensions) = self.dimensions {
                // Validate that dimensions are correct.
                match db.get(b"dimensions") {
                    // Validate that dimesions are alligned.
                    Ok(Some(value_bytes)) => {
                        let bytes = match <[u8; 4]>::try_from(value_bytes.to_vec()){
                            Ok(b) => b,
                            Err(_) => return Err(CustomError::cache_error(format!("dimensions did not have a 4 byte value.")))
                        };
                        let cached_dimensions = i32::from_le_bytes(bytes);
                        if cached_dimensions != current_dimensions {
                            return Err(CustomError::cache_error(format!(
                                "{:?} cached dimensions {} do not match current dimensions of {}.",
                                path,
                                cached_dimensions,
                                current_dimensions
                            )))
                        }
                    }
                    Ok(None) => return Err(CustomError::cache_error(format!("{:?} had no model key.", path))),
                    Err(e) => return Err(e.into()),
                }
            } else {
                // Ensure there are no dimensions used in cache.
                if let Some(_) = db.get(b"dimensions").expect("Error reading cache.") {
                    return Err(CustomError::cache_error(format!(
                        "{:?} cache contains dimensions and does not match current dimensions of None.",
                        path
                    )))
                }
            }

            // Set cache.
            self.cache = Some(db);
            Ok(self)
        } else {
            // The cache does not exist and we need to add the model and dimensions to it.
            let db = sled::open(path)?;
            // Set meta for the cache.
            let result: TransactionResult<()> = db.transaction(|tx_db| {
                tx_db.insert(b"model", self.model.as_bytes())?;
                if let Some(d) = self.dimensions {
                    let dimention_bytes = d.to_le_bytes();
                    tx_db.insert(b"dimensions", &dimention_bytes)?;
                }
                Ok(())
            });
            
            // Return error if something in the transaction failed.
            if let Err(e) = result {
                return Err(CustomError::sled_error(format!("{e:?}")));
            }
            
            // Set cache.
            self.cache = Some(db);
            Ok(self)
        }
    }

    pub fn flush(&self) {
        if let Some(db) = &self.cache {
            db.flush().expect("Error flushing DB to disk!");
        }
    }

    pub async fn get_embeddings(&self, input: Vec<impl AsRef<str>>) -> Result<Vec<ValueEmbedding>, CustomError> {
        // Define maximum number of parallel requests.
        let semaphore = Arc::new(Semaphore::new(self.parallel_requests));

        // Spawn many tasks that will send requests.
        let mut task_list = Vec::new();
        for string_to_vectorize in input {
            let string_to_vectorize = string_to_vectorize.as_ref().to_string();
            let semaphore = semaphore.clone();

            let client = self.client.clone();
            let model = self.model.clone();
            let dimensions = self.dimensions.clone();
            let cache = self.cache.clone();

            let handle = tokio::spawn(async move {
                // Acquire permit before sending request.
                let _permit = semaphore.acquire().await.unwrap();

                let key = blake3::hash(string_to_vectorize.as_bytes());

                // Check cache first
                if let Some(cache) = &cache {
                    // Check if input embedding is in cache.
                    if let Some(value) = cache.get(key.as_bytes())? {
                        // Parse cached response into string.
                        let json_string = String::from_utf8_lossy(&value);
                        // Create response from cached data.
                        let response: EmbeddingResponse = match serde_json::from_str(&json_string) {
                            Err(e) => return Err(CustomError::cache_error(format!("Error parsing cached value! {e:?}"))),
                            Ok(r) => r
                        };
                        // Return cached response.
                        return Ok(ValueEmbedding::new(
                            string_to_vectorize,
                            response
                        ));
                    }
                }

                // Get response that contains embedding
                let response = get_embedding(
                    client,
                    model,
                    dimensions,
                    &string_to_vectorize
                ).await?;

                // Set cache
                if let Some(cache) = &cache {
                    cache.insert(
                        key.as_bytes(), 
                        serde_json::json!(response)
                            .to_string()
                            .as_bytes()
                    )?;
                }

                // Drop the permit after the request has been sent.
                drop(_permit);

                Ok( ValueEmbedding::new(
                    string_to_vectorize,
                    response
                ))
            });

            task_list.push(handle);
        }

        // Collect responses from tasks.
        let mut responses = Vec::new();
        for handle in task_list {
            let response = match handle.await.expect("Join error!") {
                Ok(r) => r,
                Err(e) => return Err(e)
            };
            responses.push(response);
        }

        Ok(responses)
    }

    pub async fn get_embedding(&self, input: impl AsRef<str>) -> Result<ValueEmbedding, CustomError> {
        let key = blake3::hash(input.as_ref().as_bytes());

        if let Some(cache) = &self.cache {
            // Check if input embedding is in cache.
            if let Some(value) = cache.get(key.as_bytes())? {
                // Parse cached response into string.
                let json_string = String::from_utf8_lossy(&value);
                // Create response from cached data.
                let response: EmbeddingResponse = match serde_json::from_str(&json_string) {
                    Err(e) => return Err(CustomError::cache_error(format!("Error parsing cached value! {e:?}"))),
                    Ok(r) => r
                };
                // Return cached response.
                return Ok(
                    ValueEmbedding::new(
                        input.as_ref().to_string(),
                        response
                    )
                );
            }
        }

        let mut request = EmbeddingRequest::new(
            self.model.clone(),
            input.as_ref().to_string()
        );
        request.dimensions = self.dimensions;
        let response = self.client.embedding(request)
            .await?;
        
        if let Some(cache) = &self.cache {
            cache.insert(
                key.as_bytes(), 
                serde_json::json!(response).to_string().as_bytes()
            )?;
        }

        Ok( ValueEmbedding::new(
            input.as_ref().to_string(),
            response
        ))
    }
}
impl Drop for EmbeddingsHandler {
    fn drop(&mut self) {
        self.flush();
    }
}