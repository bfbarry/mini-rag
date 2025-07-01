// use core::error;

use ndarray::{Axis, Ix2};
use ort::{
    Error as OrtError,
    session::{Session, builder::GraphOptimizationLevel},
    value::TensorRef
};
use tokenizers::Tokenizer;
use text_splitter::{TextSplitter, ChunkConfig};
use std::{
    error::Error, f32::NEG_INFINITY, fs
};

fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Result<Vec<&str>, Box<dyn Error>> {
    // TODO return iterator instead for speed
    let conf = ChunkConfig::new(chunk_size).with_overlap(overlap)?;
    let splitter = TextSplitter::new(conf);
    let chunks = splitter.chunks(text).collect();
    Ok(chunks)
}


//https://github.com/pykeio/ort/blob/main/examples/sentence-transformers/semantic-similarity.rs
fn main() -> ort::Result<()> {
    let SAMPLE_INPUT  = fs::read_to_string("src/data/openapi_eda.json").expect("failed to read file");
    let mut embedding_session = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level1)?
        .with_intra_threads(1)?
        .commit_from_file("src/data/all-MiniLM-L6-v2.onnx")?;

    let tokenizer = Tokenizer::from_file("src/data/tokenizer.json").unwrap();
    
    let chunk_size = 1000;
    let inputs = chunk_text(&SAMPLE_INPUT, chunk_size, 200).unwrap();

    let encodings = tokenizer.encode_batch(inputs.clone(), false).map_err(|e| OrtError::new(e.to_string()))?;

    
    // flattening token IDs and masks
    let ids: Vec<i64> = encodings.iter().flat_map(|e| e.get_ids().iter().map(|i| *i as i64)).collect();
    let mask: Vec<i64> = encodings.iter().flat_map(|e| e.get_attention_mask().iter().map(|i| *i as i64)).collect();
    
    // convert above into 2d tensors
    let padded_token_length = chunk_size;
    println!("{} {} {}", encodings[0].len(), encodings[1].len(), encodings[2].len());
    let a_ids = TensorRef::from_array_view(([inputs.len(), padded_token_length], &*ids))?;
    let a_mask = TensorRef::from_array_view(([inputs.len(), padded_token_length], &*mask))?;
    // run model
    println!("here");
    let outputs = embedding_session.run(ort::inputs![a_ids, a_mask])?;

    // get embeddings tensor (2d array)
    let embeddings = outputs[1].try_extract_array::<f32>()?.into_dimensionality::<Ix2>().unwrap();
    
    println!("Sim for {}", inputs[0]);
    let query = embeddings.index_axis(Axis(0), 0);
    // let query = "create column";
    println!("{:?}", query);
    let min = NEG_INFINITY;
    for (embeddings, sentence) in embeddings.axis_iter(Axis(0)).zip(inputs.iter()).skip(1) {
        println!("HELPO");
        let dot_product: f32 = query.iter().zip(embeddings.iter()).map(|(a,b)| a * b).sum();
        println!("\t'{}': {:.1}%", sentence, dot_product*100.);
    }
    Ok(())
}