"""Offline ASR JSON-lines worker. All weights are local; stdout is protocol only."""
import argparse
import base64
import contextlib
import json
import os
from pathlib import Path
import sys
import time

os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", HF_HUB_DISABLE_TELEMETRY="1", DO_NOT_TRACK="1")
sys.stdout.reconfigure(encoding="utf-8", line_buffering=True)
sys.stdin.reconfigure(encoding="utf-8")


def emit(value):
    print(json.dumps(value, ensure_ascii=False), flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", required=True)
    parser.add_argument("--device", choices=["CPU", "GPU"], default="CPU")
    parser.add_argument("--cache", required=True)
    args = parser.parse_args()
    model = Path(args.model).resolve(strict=True)
    # Native libraries may write diagnostics; keep them away from JSON stdout.
    with contextlib.redirect_stdout(sys.stderr):
        import numpy as np
        import openvino as ov
        import openvino_genai as genai
        start = time.perf_counter()
        options = {}
        if args.device == "CPU":
            options.update(INFERENCE_NUM_THREADS=8, NUM_STREAMS="1")
        else:
            options["CACHE_DIR"] = str(Path(args.cache).resolve())
        pipe = genai.ASRPipeline(str(model), args.device, **options)
        config = pipe.get_generation_config()
        config.max_new_tokens = 256
    emit({"ready": True, "device": args.device, "version": ov.__version__, "load_seconds": time.perf_counter() - start})
    for line in sys.stdin:
        try:
            request = json.loads(line)
            if request.get("command") == "quit":
                break
            audio = np.frombuffer(base64.b64decode(request["pcm16"], validate=True), dtype="<i2").astype(np.float32) / 32768.0
            if not 1600 <= len(audio) <= 16000 * 30:
                raise ValueError("Audio must contain 0.1–30 seconds of 16kHz mono PCM")
            started = time.perf_counter()
            with contextlib.redirect_stdout(sys.stderr):
                result = pipe.generate(audio.tolist(), config)
            emit({"text": result.texts[0] if result.texts else "", "seconds": time.perf_counter() - started})
        except Exception as exc:
            emit({"error": str(exc)[:2000]})


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        emit({"error": str(exc)[:2000]})
        sys.exit(1)
