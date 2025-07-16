pub mod parser;
pub mod models;
pub mod utils;
pub mod app;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// use std::{fs};
// use std::time::Instant;
// use std::thread;
// use std::sync::mpsc;

fn main() -> ort::Result<()> {
    // tracing_subscriber::registry()
    // .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,ort=debug".into()))
    // .with(tracing_subscriber::fmt::layer())
    // .init();
    let fname = "src/data/openapi_eda.json";
    let data = parser::parse_openapi(fname).unwrap();
    // println!("{}", data);
    // return Ok(());
    // let fname = "src/data/test.txt";
    // let data  = fs::read_to_string(fname).expect("failed to read file");

    let mut rag = app::RAGBase::new(&data)?;

    // let user_input = "what url creates edarequest?";
    let user_input = "what is the name of the task that creates edarequest?";
    // let user_input = "What year was the battle?";
    
    
    // let start = Instant::now();
    let ans =  rag.query(user_input)?;
    println!("Thinking...:\n\n{}", ans);
    // println!("{:#?}", context);
    // println!("Run took: {:.2?}", start.elapsed());


    Ok(())
}