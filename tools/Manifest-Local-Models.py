"""Record the deployed, project-local model files without touching recordings or secrets."""
import datetime
import hashlib
import importlib.metadata
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
model = ROOT / "models/qwen3-asr-1.7b-openvino"
manifest_file = model / "deployment-manifest.json"
files = []
for file in sorted(model.iterdir()):
    if not file.is_file() or file == manifest_file:
        continue
    with file.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    files.append({"file": file.name, "bytes": file.stat().st_size, "sha256": digest})
versions = {name: importlib.metadata.version(name) for name in ["openvino", "openvino-genai", "openvino-tokenizers", "numpy"]}
manifest = {"created_at": datetime.datetime.now(datetime.timezone.utc).isoformat(), "source": "Qwen/Qwen3-ASR-1.7B", "source_revision": "7278e1e70fe206f11671096ffdd38061171dd6e5", "precision": "INT8 asymmetric per-channel weight compression; floating-point operations retained", "runtime": versions, "files": files, "total_bytes": sum(f["bytes"] for f in files)}
manifest_file.write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps({"files": len(files), "bytes": manifest["total_bytes"], "runtime": versions}))
