pub mod parser;
use ndarray::{Axis, Ix2};
use ort::{
    session::{builder::GraphOptimizationLevel, Session}, value::{TensorRef}, Error as OrtError
};
use tokenizers::Tokenizer;
use text_splitter::{TextSplitter, ChunkConfig};
use std::{
    error::Error
};

fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Result<Vec<&str>, Box<dyn Error>> {
    // TODO return iterator instead for speed
    let conf = ChunkConfig::new(chunk_size).with_overlap(overlap)?;
    let splitter = TextSplitter::new(conf);
    let chunks = splitter.chunks(text).collect();
    Ok(chunks)
}

fn embed(chunks: &Vec<&str>, tokenizer: &Tokenizer, embedding_session: &mut Session) -> Result<ndarray::Array2<f32>, OrtError> {
    let encodings = tokenizer.encode_batch(chunks.clone(), false)
                    .map_err(|e| OrtError::new(e.to_string()))?;

    let max_len = encodings.iter().map(|e| e.len()).max().unwrap();
    if max_len > 512 {
        return Err(OrtError::new("max_len must be < 512"));
    }
    let mut ids: Vec<i64> = Vec::new();
    let mut mask: Vec<i64> = Vec::new();

    // flattening token IDs and masks, while resizing
    for encoding in &encodings {
        let mut cur_ids :  Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let mut cur_mask:  Vec<i64> = encoding.get_attention_mask().iter().map(|&m| m as i64).collect();

        cur_ids.resize(max_len, 0);
        cur_mask.resize(max_len, 0);

        ids.extend(cur_ids);
        mask.extend(cur_mask);
    }

    // convert above into 2d tensors
    let padded_token_length = max_len;
    let shape = [chunks.len(), padded_token_length];

    let a_ids   = TensorRef::from_array_view((shape, &*ids))?;
    let a_mask  = TensorRef::from_array_view((shape, &*mask))?;

    // run model
    println!("here");
    let outputs = embedding_session.run(ort::inputs![a_ids, a_mask])?;

    println!("jhere2");
    // get embeddings tensor (2d array)
    let embeddings = outputs[1].try_extract_array::<f32>()?.into_dimensionality::<Ix2>().unwrap();
    
    Ok(embeddings.into_owned()) 
}

fn find_sim_in_range(query_vec: &ndarray::Array1<f32>, 
                     embeddings: &ndarray::Array2<f32>, 
                     chunks: &Vec<&str>,
                     ix0: usize, ix1: usize,
                    ) -> Result<(f32, String), OrtError> 
{
    let mut max_cos = 0.;
    let mut argmax = "";

    for (embeddings, sentence) in embeddings.axis_iter(Axis(0)).zip(chunks.iter()).skip(ix0).take(ix1-ix0) {
        let dot_product: f32 = query_vec.iter().zip(embeddings.iter()).map(|(a,b)| a * b).sum();
        if dot_product > max_cos {
            max_cos = dot_product;
            argmax = sentence;
        }

        // println!("{:.1}%", dot_product*100.);
    }

    Ok((max_cos, argmax.to_string()))    
}
//https://github.com/pykeio/ort/blob/main/examples/sentence-transformers/semantic-similarity.rs
fn main() -> ort::Result<()> {
    let fname = "src/data/openapi_eda.json";
    // let fname = "src/data/test.txt";
    // let sample_input  = fs::read_to_string(fname).expect("failed to read file");
    let sample_input = parser::parse_openapi(fname).unwrap();

    let mut embedding_session = Session::builder()?
                                    .with_optimization_level(GraphOptimizationLevel::Level3)?
                                    // .with_intra_threads(1)?
                                    .commit_from_file("src/data/all-MiniLM-L6-v2.onnx")?;

    let tokenizer = Tokenizer::from_file("src/data/tokenizer.json").unwrap();
    
    let chunk_size = 700;
    let chunks = chunk_text(&sample_input, chunk_size, 200).unwrap();

    let embeddings = embed(&chunks, &tokenizer, &mut embedding_session).unwrap();
    
    let query2 = "what route updates the retriever";
    let query_embeddings = embed(&vec![query2], &tokenizer, &mut embedding_session).unwrap();
    let query_vec = query_embeddings.index_axis(Axis(0), 0).into_owned();
    
    let n_chunks = chunks.len();
    let step = n_chunks/5;

    let (c1, s1) = find_sim_in_range(&query_vec, &embeddings, &chunks, 0, step)?;
    let (c2, s2) = find_sim_in_range(&query_vec, &embeddings, &chunks, step, step*2)?;
    let (c3, s3) = find_sim_in_range(&query_vec, &embeddings, &chunks, step*2, step*3)?;
    let (c4, s4) = find_sim_in_range(&query_vec, &embeddings, &chunks, step*3, step*4)?;
    let (c5, s5) = find_sim_in_range(&query_vec, &embeddings, &chunks, step*4, n_chunks)?;

    println!("Argmaxes are >>>{}\n\n>>>{}\n\n>>>{}\n\n>>>{}\n\n>>>{}",
         s1, s2, s3, s4, s5);
    println!("scores are {} {} {} {} {}", c1, c2, c3, c4, c5);
    Ok(())
}