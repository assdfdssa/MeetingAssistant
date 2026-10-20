"""Development-time download of pinned official Qwen weights; never called by the app."""
import concurrent.futures
import argparse
import hashlib
import json
from pathlib import Path
import urllib.request

ROOT = Path(__file__).resolve().parent.parent
REVISIONS = {"1.7B": "7278e1e70fe206f11671096ffdd38061171dd6e5"}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--size", choices=list(REVISIONS), default="1.7B")
    size = parser.parse_args().size
    REPO = f"Qwen/Qwen3-ASR-{size}"
    REVISION = REVISIONS[size]
    DEST = ROOT / "models" / "sources" / f"Qwen3-ASR-{size}"
    DEST.mkdir(parents=True, exist_ok=True)
    info = json.load(urllib.request.urlopen(f"https://huggingface.co/api/models/{REPO}/revision/{REVISION}?blobs=true"))
    def download(item):
        name = item["rfilename"]
        if name.startswith("."):
            return
        target = DEST / name
        size = item["size"]
        expected = item.get("lfs", {}).get("sha256")
        if not target.is_file() or target.stat().st_size != size:
            part = target.with_suffix(target.suffix + ".download")
            with urllib.request.urlopen(f"https://huggingface.co/{REPO}/resolve/{REVISION}/{name}", timeout=90) as response, part.open("wb") as output:
                while chunk := response.read(8 * 1024 * 1024):
                    output.write(chunk)
            if part.stat().st_size != size:
                raise RuntimeError(f"Incomplete download: {name}")
            part.replace(target)
        digest = hashlib.file_digest(target.open("rb"), "sha256").hexdigest()
        if expected and digest != expected:
            raise RuntimeError(f"SHA256 mismatch: {name}")
        print(json.dumps({"file": name, "bytes": size, "sha256": digest}), flush=True)
        return {"file": name, "bytes": size, "sha256": digest}
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        files = list(filter(None, pool.map(download, info["siblings"])))
    (DEST / "source-manifest.json").write_text(json.dumps({"repository": REPO, "revision": REVISION, "files": files}, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
