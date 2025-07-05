use text_splitter::{TextSplitter, ChunkConfig};
use std::error::Error;

pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Result<Vec<String>, Box<dyn Error>> {
    // TODO return iterator instead for speed
    let conf = ChunkConfig::new(chunk_size).with_overlap(overlap)?;
    let splitter = TextSplitter::new(conf);
    let chunks = splitter.chunks(text).map(|s| s.to_string()).collect();
    Ok(chunks)
}