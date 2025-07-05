pub mod parser;
pub mod models;
pub mod utils;
pub mod app;

use std::{fs};
// use std::time::Instant;
// use std::thread;
// use std::sync::mpsc;

fn main() -> ort::Result<()> {
    // let fname = "src/data/openapi_eda.json";
    // let sample_input = parser::parse_openapi(fname).unwrap();

    let fname = "src/data/test.txt";
    let data  = fs::read_to_string(fname).expect("failed to read file");

    let mut rag = app::RAG::new(&data)?;

    let user_input = "where did it take place?";
    
    
    // let start = Instant::now();
    let ans =  rag.query(user_input)?;
    println!("{}", ans);
    // println!("{:#?}", context);
    // println!("Run took: {:.2?}", start.elapsed());


    Ok(())
}