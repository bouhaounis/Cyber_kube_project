import os
import numpy as np
import onnx
from onnx import helper, TensorProto


def make_synth(n=5000, in_dim=16, anomaly_ratio=0.05, seed=42):
    rng = np.random.default_rng(seed)
    normal = rng.normal(0, 1.0, size=(n, in_dim)).astype(np.float32)
    anomalies = rng.normal(0, 3.5, size=(int(n * anomaly_ratio), in_dim)).astype(np.float32)
    x = np.concatenate([normal, anomalies], axis=0)
    y = np.concatenate([np.zeros(len(normal)), np.ones(len(anomalies))], axis=0).astype(np.int64)
    idx = rng.permutation(len(x))
    return x[idx], y[idx]


def main():
    out_dir = os.environ.get("CYBERKUBE_ML_OUT", "ml-models/artifacts")
    os.makedirs(out_dir, exist_ok=True)

    in_dim = 16
    x, y = make_synth(in_dim=in_dim)

    # Create a tiny ONNX model without PyTorch (Windows-friendly):
    # reconstruction = Identity(features)
    # This is enough to demo the pipeline + an anomaly score can be computed as ||x - recon|| in Rust/Go/Python.
    input_tensor = helper.make_tensor_value_info("features", TensorProto.FLOAT, ["N", in_dim])
    output_tensor = helper.make_tensor_value_info("reconstruction", TensorProto.FLOAT, ["N", in_dim])
    node = helper.make_node("Identity", inputs=["features"], outputs=["reconstruction"])
    graph = helper.make_graph(
        nodes=[node],
        name="cyberkube_anomaly_demo",
        inputs=[input_tensor],
        outputs=[output_tensor],
    )
    model = helper.make_model(graph, producer_name="cyber-kube")
    onnx.checker.check_model(model)

    onnx_path = os.path.join(out_dir, "anomaly_detection.onnx")
    onnx.save(model, onnx_path)
    np.save(os.path.join(out_dir, "dataset_x.npy"), x)
    np.save(os.path.join(out_dir, "dataset_y.npy"), y)
    print(f"saved: {onnx_path}")


if __name__ == "__main__":
    main()

