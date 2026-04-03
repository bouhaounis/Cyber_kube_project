## ML Models (demo)

Cette partie sert à **générer un modèle ONNX** de démonstration (`anomaly_detection.onnx`) à partir d’un dataset synthétique.

### Pré-requis
- Python **64-bit** (important sur Windows)
- `pip` à jour recommandé

### Run

```powershell
cd ml-models
python -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install --upgrade pip
pip install -r requirements.txt
python training\train_anomaly.py
```

Artefacts : `ml-models/artifacts/`

