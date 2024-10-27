use crate::worldgen::Chunk;
use bincode::deserialize;
use reqwest::Error;
pub async fn fetch_chunk(x: u32, y: u32, index: u32) -> Result<Chunk, Error> {
    let url = format!("http://localhost:3000/chunk?x={}&y={}&index={}", x, y, index);
    let response = reqwest::get(&url).await?;

    // Parse the JSON response to a Chunk struct
    let chunk = response.bytes().await?;
    let chunk_des = bincode::deserialize(&chunk).unwrap();
    Ok(chunk_des)
}
