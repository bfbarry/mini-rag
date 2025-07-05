use crate::models;
use crate::utils;
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
    pub fn new(data: &str) -> Result<Self, OrtError> {
        // todo customize these with 
        let mut embedding_model = models::EmbeddingModel::new()?;


        let chunk_size = 700;
        let chunks = utils::chunk_text(&data, chunk_size, 200).unwrap();
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

        let mut max_cos = 0.;
        let mut argmax = "";

        let mut ni: usize = 1;
        let mut res: Vec<(f32, &str)> = vec![];

        for (embeddings, sentence) in self.embedding_model.embeddings.axis_iter(Axis(0)).zip(self.chunks.iter()) {
            let dot_product: f32 = query_vec.iter().zip(embeddings.iter()).map(|(a,b)| a * b).sum();
            if dot_product > max_cos {
                max_cos = dot_product;
                argmax = sentence;
            }
            if ni % n == 0 {
                res.push((max_cos, argmax));
                max_cos = 0.;
                argmax = "";
            }
            ni += 1;
        }
        res.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        Ok(res)    
    }    
}


pub struct RAG {
    bert: models::BertModel,
    vec_db: VecDB
}

impl RAG {
    pub fn new(data_fpath: &str) -> Result<Self, OrtError> {
        let bert = models::BertModel::new()?;
        let vec_db = VecDB::new(data_fpath)?;
        Ok(Self {
            bert,
            vec_db,
        })
    }

    pub fn query(&mut self, user_input: &str) -> Result<String, OrtError> {
        
        let topn = self.vec_db.find_top_n_sim(user_input, 5)?;
        let mut context = "Context for this question:\n\n".to_string();
        for (score, contents) in topn {
            context.push_str(&format!("score: {}\ncontents: {}\n\n", score, contents));
        }
        let input = format!("{} {}", user_input, context);
        let (ids, mask) = self.bert.encode(input)?;
        let (start_logits, end_logits) = self.bert.forward(ids, mask)?;

        let answer = self.bert.decode(start_logits, end_logits)?;
        Ok(answer)

    }
}