use ort::{
    session::{builder::GraphOptimizationLevel, Session}, value::{Tensor, TensorValueType, Value}, Error as OrtError
};
use tokenizers::{Tokenizer};
use ndarray::{Ix2, IxDynImpl, Dim};
pub struct ModelBase  {
    ort_session: Session,
    tokenizer: Tokenizer,
}

pub struct EmbeddingModel {
    modelbase: ModelBase,
    pub embeddings: ndarray::Array2<f32>,
}

pub struct BertModel {
    modelbase: ModelBase,
    input_ids: Vec<i64>,
}

pub trait ModelI <'a> where Self: Sized {
    type EncodeInput;
    type OutputTensor;
    fn new() -> Result<Self, OrtError> ;
    fn forward(&mut self, ids: Value<TensorValueType<i64>>, mask: Value<TensorValueType<i64>>) -> Result<Self::OutputTensor, OrtError>;
    fn encode(&mut self, input: Self::EncodeInput) -> Result<(Value<TensorValueType<i64>>, Value<TensorValueType<i64>>), OrtError>;    
}


impl ModelBase {
    fn new(onnx_path: &str, tok_path: &str) -> Result<Self, OrtError> {
        let sess = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            // .with_intra_threads(1)?
            .commit_from_file(onnx_path)?;

        let tokenizer = Tokenizer::from_file(tok_path).unwrap();
        
        Ok(Self {
            ort_session: sess,
            tokenizer  : tokenizer
        })
    }
}


impl<'a> ModelI<'a> for EmbeddingModel {
    type EncodeInput = &'a Vec<String>;
    type OutputTensor = ndarray::Array2<f32>;

    fn new() -> Result<Self, OrtError> {
        let modelbase = ModelBase::new(
            "src/models/all-MiniLM-L6-v2.onnx", 
            "src/models/minilm-tokenizer.json"
            )?;

        let embeddings = ndarray::Array2::zeros((0, 0));

        Ok(Self {
            modelbase,
            embeddings
        })
    }

    fn encode(&mut self, input: Self::EncodeInput) -> Result<(Value<TensorValueType<i64>>, Value<TensorValueType<i64>>), OrtError> {
        let encodings = self.modelbase.tokenizer.encode_batch(input.clone(), false)
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
        let shape = [input.len(), padded_token_length];

        let a_ids   = Tensor::from_array((shape, ids))?;
        let a_mask  = Tensor::from_array((shape, mask))?;
        Ok((a_ids, a_mask))
    }
    
    fn forward(&mut self, 
                ids: Value<TensorValueType<i64>>, 
                mask: Value<TensorValueType<i64>>) -> Result<Self::OutputTensor, OrtError> {
        // run model
        let outputs = self.modelbase.ort_session.run(ort::inputs![ids, mask])?;

        // get embeddings tensor (2d array)
        let embeddings = outputs[1].try_extract_array::<f32>()?
                                   .into_dimensionality::<Ix2>().unwrap();

        Ok(embeddings.into_owned())
    }
}

impl EmbeddingModel {
    pub fn set_embeddings(&mut self, chunks: &Vec<String>) {
        let (a_ids, a_mask) = self.encode(chunks).unwrap();
        let embeddings = self.forward(a_ids, a_mask).unwrap();
        self.embeddings = embeddings;
    }
}


type BertLogits = ndarray::Array<f32, Dim<IxDynImpl>>;
impl <'a> ModelI <'a> for BertModel {
    type EncodeInput = String;
    type OutputTensor = (BertLogits, BertLogits);

    fn new() -> Result<Self, OrtError> {
        let modelbase = ModelBase::new(
            "src/models/onnx_distilbert_qa/model.onnx",
            "src/models/onnx_distilbert_qa/tokenizer.json"
            )?;
        let input_ids = vec![];

        Ok (Self {
            modelbase,
            input_ids,
        })
        
    } 

    fn encode(&mut self, input: Self::EncodeInput) -> Result<(Value<TensorValueType<i64>>, 
                                                          Value<TensorValueType<i64>>), 
                                                          OrtError> {
        let encoding = self.modelbase.tokenizer.encode(input, true)?;

        // Required inputs for BERT-style QA:
        let input_ids: Vec<i64> = encoding.get_ids().to_vec().iter().map(|&e| e as i64).collect();
        self.input_ids = input_ids.clone(); // TODO: try not to clone?
        
        let attention_mask: Vec<i64> = encoding.get_attention_mask().to_vec().iter().map(|&e| e as i64).collect();
        
        // 4. Convert to ONNX tensors
        let a_ids =  Tensor::from_array(([1, input_ids.len()], input_ids))?; 
        let a_mask =  Tensor::from_array(([1, attention_mask.len()], attention_mask))?;
        
        Ok((a_ids, a_mask))
    }

    fn forward(&mut self, ids: Value<TensorValueType<i64>>, 
                          mask: Value<TensorValueType<i64>>) 
                -> Result<Self::OutputTensor, OrtError> {
        let outputs = self.modelbase.ort_session.run(ort::inputs![ids, mask])?;
        let start_logits  = outputs[0].try_extract_array::<f32>()?.into_owned();
        let end_logits  = outputs[1].try_extract_array::<f32>()?.into_owned();

        Ok((start_logits, end_logits))        
    }
}

impl BertModel {
    pub fn decode(&self, start_logits: BertLogits, 
                     end_logits: BertLogits) -> Result<String, OrtError> {
        let start = start_logits.index_axis(ndarray::Axis(0), 0).iter().cloned()
                                .enumerate().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap().0;
        let end   = end_logits.index_axis(ndarray::Axis(0), 0).iter().cloned()
                                .enumerate().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap().0;
    
        let ids = &self.input_ids[start..=end];
        let ids_u32: Vec<u32> = ids.iter().map(|&e| e as u32).collect();
        let answer = self.modelbase.tokenizer.decode(&ids_u32, true)?;

        Ok(answer)
    }
}


