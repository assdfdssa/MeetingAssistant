"""Local bridge used by the connected GitHub uploader; never reads data/ or models/."""
import base64
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent.parent
manifest = json.loads((ROOT / "build/github-source-manifest.json").read_text(encoding="utf-8"))
mode = sys.argv[1]
if mode == "manifest":
    print(json.dumps(manifest, ensure_ascii=False))
elif mode == "binary":
    files = [f for f in manifest["files"] if f["binary"]]
    file = files[int(sys.argv[2])]
    content = base64.b64encode((Path(manifest["root"]) / file["path"]).read_bytes()).decode()
    if len(sys.argv) > 3:
        start, length = int(sys.argv[3]), int(sys.argv[4])
        content = content[start:start+length]
    print(json.dumps({**file, "content": content}, ensure_ascii=False))
elif mode == "text":
    files = [f for f in manifest["files"] if not f["binary"]]
    start, count = int(sys.argv[2]), int(sys.argv[3])
    print(json.dumps([{"path": f["path"], "mode": f["mode"], "type": "blob", "content": (Path(manifest["root"]) / f["path"]).read_bytes().decode("utf-8")} for f in files[start:start+count]], ensure_ascii=False))
else:
    raise ValueError("Unknown mode")
