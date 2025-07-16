import onnx
model = onnx.load("src/models/onnx_roberta_qa/model.onnx")
for input in model.graph.input:
    print(input.name)