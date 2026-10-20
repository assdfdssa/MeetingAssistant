"""Real offline inference smoke test on locally synthesized, non-user samples."""
import argparse
import base64
import json
from pathlib import Path
import subprocess
import sys
import threading
import time
import wave

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--device", choices=["CPU", "GPU"], default="CPU")
    parser.add_argument("--size", choices=["1.7B"], default="1.7B")
    args = parser.parse_args()
    report = {"device": args.device, "size": args.size, "samples": [], "cloud_calls": False}
    log = (ROOT / f"logs/openvino-asr-{args.size}-{args.device}.log").open("w", encoding="utf-8")
    process = subprocess.Popen([str(ROOT / "tools/openvino-python/python.exe"), "-u", str(ROOT / "tools/openvino-asr.py"), "--model", str(ROOT / f"models/qwen3-asr-{args.size.lower()}-openvino"), "--device", args.device, "--cache", str(ROOT / "models/openvino-cache")], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=log, text=True, encoding="utf-8", creationflags=subprocess.CREATE_NO_WINDOW)
    watchdog = threading.Timer(600, process.kill)
    watchdog.daemon = True
    watchdog.start()
    try:
        ready = json.loads(process.stdout.readline())
        assert ready.get("ready"), ready
        report["startup"] = ready
        for name in ["chinese", "english", "mixed"]:
            with wave.open(str(ROOT / f"build/asr-samples/{name}.wav"), "rb") as wav:
                assert wav.getframerate() == 16000 and wav.getnchannels() == 1 and wav.getsampwidth() == 2
                audio = wav.readframes(wav.getnframes())
            start = time.perf_counter()
            segments = []
            # Match the desktop adapter's approximate 8-second dispatch size.
            for offset in range(0, len(audio), 32000 * 8):
                part = audio[offset:offset + 32000 * 8]
                if len(part) < 3200:
                    part += bytes(3200 - len(part))
                process.stdin.write(json.dumps({"pcm16": base64.b64encode(part).decode()}) + "\n")
                process.stdin.flush()
                segment = json.loads(process.stdout.readline())
                assert "text" in segment, segment
                segments.append(segment)
            result = {"text": " ".join(s["text"] for s in segments), "segments": segments}
            assert result["text"].strip(), result
            result.update(name=name, audio_seconds=len(audio) / 32000, wall_seconds=time.perf_counter() - start)
            result["rtf"] = result["wall_seconds"] / result["audio_seconds"]
            report["samples"].append(result)
            print(json.dumps(result, ensure_ascii=False), flush=True)
        zh, en, mixed = [r["text"] for r in report["samples"]]
        assert "概率" in zh and "报告" in zh, zh
        assert "probability" in en.lower() and "report" in en.lower(), en
        assert "概率" in mixed and ("bayes" in mixed.lower() or "贝叶斯" in mixed), mixed
        report["passed"] = True
    finally:
        watchdog.cancel()
        process.kill()
        process.wait()
        log.close()
        (ROOT / f"logs/openvino-asr-{args.size}-{args.device}.json").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
