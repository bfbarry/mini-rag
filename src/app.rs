use crate::models;
use crate::utils;
use crate::parser;
use models::ModelI;
use ndarray::{Axis};

use ort::{
    Error as OrtError
};

pub struct VecDB {
    embedding_model: models::EmbeddingModel,
    chunks: Vec<String>
}

impl VecDB {
    pub fn new(data: &str, chunk_size: usize, chunks: usize) -> Result<Self, OrtError> {
        // todo customize these with 
        let mut embedding_model = models::EmbeddingModel::new()?;


        let chunks = utils::chunk_text(&data, chunk_size, chunks).unwrap();
        println!("SETTING EMBEDDINGS...");
        embedding_model.set_embeddings(&chunks);
        println!("DONE...");

        Ok(Self {
            embedding_model,
            chunks
        })
    }

    pub fn find_top_n_sim(&mut self, 
                       query: &str, 
                       n: usize, 
       ) -> Result<Vec<(f32, &str)>, OrtError> {
        
        let (ids, mask) = self.embedding_model.encode(&vec![query.to_string()])?;
        let query_embeddings = self.embedding_model.forward(ids, mask).unwrap();
        let query_vec = query_embeddings.index_axis(Axis(0), 0).into_owned();

        let mut res: Vec<(f32, &str)> = vec![];
        for (embeddings, sentence) in self.embedding_model.embeddings.axis_iter(Axis(0)).zip(self.chunks.iter()) {
            let dot_product: f32 = query_vec.iter().zip(embeddings.iter()).map(|(a,b)| a * b).sum();
            res.push((dot_product, sentence));
        }
        res.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        Ok(res[..n].to_vec())    
    }    
}


pub struct RAGBase {
    bert: models::BertModel,
    vec_db: VecDB
}

// do I need this?
enum AgentSpecialization {
    OpenAPI,
    Codebase, // type converter etc
    Educational, // e.g., a programming primitives "how to model composition in Rust, how to start a thread"
    Debugger // accesses a stream of logging
}

// "Agent" in the sense that it has functionality beyond LLM io
pub trait AgentI where {
    fn execute(&mut self, user_input: &str) -> Result<String, OrtError>;

}

pub struct OpenAPIAgent {
    rag_base: RAGBase
}


pub struct CodebaseAgent {
    rag_base: RAGBase
}

pub struct AgentBuilder {
    // this class constructs a different agent depending on user input 
}

impl RAGBase {
    pub fn new(data: &str, chunk_size: usize, chunks: usize) -> Result<Self, OrtError> {
        let bert = models::BertModel::new()?;
        let vec_db = VecDB::new(data, chunk_size, chunks)?;
        Ok(Self {
            bert,
            vec_db,
        })
    }

    pub fn query(&mut self, user_input: &str) -> Result<String, OrtError> {
        
        let topn = self.vec_db.find_top_n_sim(user_input, 4)?;

        // let mut context = "(A URL is another word for name)".to_string();
        let mut context = String::new();
        // let mut context = "Answer with the route: \n\n".to_string();
        for (score, contents) in topn {

            // println!("{} {}", score, contents);
            // context.push_str(&format!("{}", contents));
            context.push_str(&format!("[{:.1}% match]\n{}\n\n",score*100., contents));
        }

        Ok(context)

    }

    fn llm_answer_UNSTABLE(&mut self, user_input: &str) -> Result<String, OrtError> {
        let context = self.query(user_input)?;
        // context = "The battle of Fuck occurred on March 2 1892".to_string();
        let input = format!("Question: {} \nContext:\n{}", user_input, context);
        println!("{}", input);
        let (ids, mask) = self.bert.encode(input)?;
        let (start_logits, end_logits) = self.bert.forward(ids, mask)?;
        // println!("Hello?");

        let answer = self.bert.decode(start_logits, end_logits)?;
        Ok(answer)

    }
}
impl OpenAPIAgent {
    pub fn new(fname: &str) -> Result<Self, OrtError> {
        let data = parser::parse_openapi(fname).unwrap();
        let rag_base = RAGBase::new(&data, 150, 70)?;
        Ok(Self { rag_base})
    }

}

impl AgentI for OpenAPIAgent {

    fn execute(&mut self, user_input: &str) -> Result<String, OrtError> {
        // TODO: should store method, path, summary separately to do cool text formatting after
        //\x1B[1m{BOLD METHOD}\x1B[0m 
        // \x1B[4m{UNDERLINED URL}\x1B[0m

        Ok(self.rag_base.query(user_input)?)
    }
}

impl CodebaseAgent{
    pub fn new(root_path: &str) -> Result<Self, OrtError> {
        let data = parser::parse_codebase(root_path).unwrap();
        let rag_base = RAGBase::new(&data, 250, 100)?;
        Ok (
            Self {
                rag_base
            }
        )
    }
}
impl AgentI for CodebaseAgent {

    fn execute(&mut self, user_input: &str) -> Result<String, OrtError> {
        Ok("great".to_string())
    }
}