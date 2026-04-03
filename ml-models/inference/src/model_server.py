"""Model serving API for Cyber-Kube ML models"""

from flask import Flask, request, jsonify
import numpy as np
import onnxruntime as ort
import os

app = Flask(__name__)

# Load ONNX model
model_path = os.getenv("MODEL_PATH", "artifacts/anomaly_detection.onnx")
session = ort.InferenceSession(model_path)

@app.route("/predict", methods=["POST"])
def predict():
    """Predict anomaly score"""
    data = request.json
    features = np.array(data["features"], dtype=np.float32).reshape(1, -1)
    
    input_name = session.get_inputs()[0].name
    output = session.run(None, {input_name: features})[0]
    
    score = float(output[0][0])
    is_anomaly = score > 0.5
    
    return jsonify({
        "score": score,
        "is_anomaly": is_anomaly,
        "threshold": 0.5
    })

@app.route("/health", methods=["GET"])
def health():
    return jsonify({"status": "ok"})

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8000)
