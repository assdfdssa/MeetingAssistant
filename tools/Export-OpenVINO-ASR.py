"""Development-only conversion of pinned official local weights (no runtime downloads)."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
os.environ.update(HF_HOME=str(ROOT / "tools/hf-cache"), HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", HF_HUB_DISABLE_TELEMETRY="1", DO_NOT_TRACK="1", TOKENIZERS_PARALLELISM="false", PYTHONUNBUFFERED="1")
parser = argparse.ArgumentParser()
parser.add_argument("--size", choices=["1.7B"], default="1.7B")
args = parser.parse_args()
source = ROOT / f"models/sources/Qwen3-ASR-{args.size}"
destination = ROOT / f"models/qwen3-asr-{args.size.lower()}-openvino"
if not (source / "source-manifest.json").is_file():
    raise RuntimeError("Download and verify pinned official source weights first")
destination.mkdir(parents=True, exist_ok=True)
command = [str(Path(sys.executable).with_name("optimum-cli.exe")), "export", "openvino", "--model", str(source), "--task", "automatic-speech-recognition-with-past", "--trust-remote-code", "--weight-format", "int8", str(destination)]
print("Converting local source to OpenVINO INT8:", args.size, flush=True)
subprocess.run(command, check=True, env=os.environ, cwd=ROOT)
# Source config includes a sampling-only temperature while do_sample=False.
# Preserve deterministic decoding, without editing the verified source files.
generation = json.loads((source / "generation_config.json").read_text(encoding="utf-8"))
if not generation.get("do_sample", False):
    generation.pop("temperature", None)
(destination / "generation_config.json").write_text(json.dumps(generation, indent=2), encoding="utf-8")
print(json.dumps({"source": str(source), "destination": str(destination), "precision": "int8"}), flush=True)
