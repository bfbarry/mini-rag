#!/bin/bash


# as of now need a venv with low numpy version
python3 -m venv qa_venv
pip_=/Users/brianbarry/Desktop/ucsd-its/mini_rag/qa_env/bin/pip
$pip install transformers onnx onnxruntime optimum "numpy<2.0"


mkdir src/models
cd src/models
# embedder
wget https://cdn.pyke.io/0/pyke:ort-rs/example-models@0.0.0/all-MiniLM-L6-v2.onnx
# NOTE: tokenizer downloaded from hugginface as standalone json...
# TODO: should be getting the model and tokenizer form https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2
# and then use a python script to export
# BERT
optimum-cli export onnx --model distilbert/distilbert-base-cased-distilled-squad onnx_distilbert_qa/ --task question-answering
