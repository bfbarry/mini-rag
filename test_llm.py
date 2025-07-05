from transformers import AutoTokenizer
import onnxruntime as ort
import numpy as np

# Load tokenizer
tokenizer = AutoTokenizer.from_pretrained("src/models/onnx_distilbert_qa")

# Define QA input
question = "Where does the sun rise?"
context = "The sun rises in the east and sets in the west."

# Tokenize and ensure correct numpy dtype
inputs = tokenizer(question, context, return_tensors="np")
inputs = {k: v.astype(np.int64) for k, v in inputs.items()}

# Load ONNX model
session = ort.InferenceSession("src/models/onnx_distilbert_qa/model.onnx")

# Run model
start_logits, end_logits = session.run(None, inputs)

# Extract answer span
start = np.argmax(start_logits[0])
end = np.argmax(end_logits[0]) + 1
answer_ids = inputs["input_ids"][0][start:end]
answer = tokenizer.decode(answer_ids)

print("Answer:", answer)
