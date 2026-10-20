"""Prepare a source-only snapshot. Never modifies the working Meetily checkout."""
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "build/github-source-snapshot"
SOURCE = ROOT / "meetily"
OUT.mkdir(parents=True, exist_ok=True)
selected = {}
gitlinks = []
excluded = []


def add(source, target, mode="100644"):
    source = Path(source)
    if source.is_symlink():
        raise RuntimeError("Unexpected symbolic link: " + target)
    if not source.is_file():
        raise RuntimeError("Missing source file: " + target)
    if source.suffix.lower() in {".mp4", ".mov", ".webm", ".wav", ".mp3", ".exe", ".dll", ".pdb", ".onnx", ".gguf", ".safetensors"} or (source.suffix.lower()==".gif" and source.stat().st_size>2_000_000):
        excluded.append(target)
        return
    if source.name.startswith(".env") and source.name not in {".env.example", ".env.template"}:
        excluded.append(target)
        return
    raw = source.read_bytes()
    if raw.startswith(b"version https://git-lfs.github.com/spec/"):
        raise RuntimeError("LFS pointer needs review: " + target)
    binary = b"\0" in raw
    content = None
    if not binary:
        try:
            content = raw.decode("utf-8")
        except UnicodeDecodeError:
            binary = True
    if content is not None:
        # Export sanitization only: remove a real course identifier and this PC's login.
        content = content.replace("00000000-0000-4000-8000-000000000001", "00000000-0000-4000-8000-000000000001")
        content = content.replace("00000000-0000-4000-8000-000000000002", "00000000-0000-4000-8000-000000000002")
        content = content.replace("YOUR_USER", "YOUR_USER").replace("YOUR_USER", "YOUR_USER")
        if target.startswith(("docs/", "dist/")):
            content = content.replace("研究方法", "示例课程")
        patterns = [r"\bsk-[A-Za-z0-9_-]{20,}\b", r"\bgh[pousr]_[A-Za-z0-9]{20,}\b", r"\bgithub_pat_[A-Za-z0-9_]{20,}\b", r"\bAKIA[A-Z0-9]{16}\b", r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"]
        if any(re.search(pattern, content) for pattern in patterns):
            raise RuntimeError("Potential credential in " + target)
        raw = content.encode("utf-8")
    file = OUT / target
    file.parent.mkdir(parents=True, exist_ok=True)
    file.write_bytes(raw)
    selected[target] = {"path": target, "mode": mode, "type": "blob", "binary": binary, "bytes": len(raw), "git_sha": hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()}


tracked = subprocess.check_output(["git", "-c", f"safe.directory={SOURCE.as_posix()}", "-C", str(SOURCE), "ls-files", "-s", "-z"])
for entry in tracked.split(b"\0"):
    if not entry:
        continue
    meta, path = entry.split(b"\t", 1)
    mode, sha, stage = meta.decode().split()
    name = path.decode("utf-8")
    if stage != "0":
        raise RuntimeError("Resolve merge conflicts before publishing")
    if mode == "160000":
        gitlinks.append({"path": "meetily/" + name, "mode": mode, "type": "commit", "sha": sha})
    else:
        add(SOURCE / name, "meetily/" + name, mode)

for folder in ["frontend/src-tauri/src/classcatch", "frontend/src/components/classcatch"]:
    for file in (SOURCE / folder).rglob("*"):
        if file.is_file():
            add(file, "meetily/" + file.relative_to(SOURCE).as_posix())
add(SOURCE / "frontend/scripts/build-classroom.cjs", "meetily/frontend/scripts/build-classroom.cjs")
for name in ["README.md", "启动桌面版.cmd", "预览界面.cmd"]:
    add(ROOT / name, name)
for folder in ["docs", "licenses"]:
    for file in (ROOT / folder).iterdir():
        if file.is_file() and file.suffix.lower() in {".md", ".txt", ".json"}:
            add(file, folder + "/" + file.name)
for name in ["开始使用.md", "build-info.json"]:
    add(ROOT / "dist" / name, "dist/" + name)
for file in (ROOT / "tools").iterdir():
    if file.is_file() and file.suffix.lower() in {".ps1", ".py", ".cjs", ".json", ".txt"}:
        add(file, "tools/" + file.name)
for folder, directories, files in os.walk(ROOT / "tests"):
    directories[:] = [d for d in directories if d not in {"target", "node_modules", "__pycache__", ".git"}]
    for name in files:
        file = Path(folder) / name
        if name == "native-repaired-course-view.cjs":
            excluded.append(file.relative_to(ROOT).as_posix())
            continue
        if file.suffix.lower() in {".cjs", ".py", ".ps1", ".rs", ".toml", ".lock"}:
            add(file, file.relative_to(ROOT).as_posix())

extra = {
    ".gitignore": """# Local recordings, credentials and personal history
/data/
/data-backups/
/logs/
**/.env*
!**/.env.example
!**/.env.template
**/*.pem
**/*.key
# Model weights, runtimes and compiled outputs
/models/
/build/
**/node_modules/
**/target/
**/.next*/
**/out/
**/__pycache__/
**/*.pyc
**/*.exe
**/*.dll
**/*.pdb
**/*.gguf
**/*.onnx
**/*.safetensors
**/*.sqlite*
**/*.db
**/*.wav
**/*.mp4
**/*.mp3
**/*.mov
**/*.tsbuildinfo
/tools/*/
/tools/*.tgz
/tools/github-sync.local.json
/meetily/frontend/src-tauri/binaries/
""",
    ".gitmodules": """[submodule "meetily/backend/whisper.cpp"]
    path = meetily/backend/whisper.cpp
    url = https://github.com/Zackriya-Solutions/whisper.cpp
    branch = develop
""",
    "docs/GitHub源码使用说明.md": """# GitHub源码快照

此仓库保存课堂补听助手源码、文档、脚本、测试和第三方许可。Meetily源码放在meetily/，保留其上游说明和MIT许可。本项目基于Meetily Community v0.4.1，上游提交a2cb62e827da7ef59f65064c97233efb2313878e。

录音、转写数据库、API密钥、诊断日志、安装包、本地模型、Python/Node/Rust工具链和构建缓存不在仓库里。dist/只有交付说明和构建校验记录，没有可直接下载的安装包。源代码拉取后不能直接双击启动脚本运行，必须准备开发环境并构建。

推荐拉取命令：git clone --recurse-submodules https://github.com/assdfdssa/MeetingAssistant.git 。根目录.gitmodules固定whisper.cpp子模块关系。

Windows开发需要Node/pnpm、Rust、MSVC C++与Windows SDK、CMake/Ninja/Clang等，并准备Tauri所需FFmpeg、ONNX Runtime与release版llama-helper。tools/Build-Environment.ps1和部分测试脚本对应原开发机目录，迁移时请按实际目录修改，不把已验证的本机环境误当作通用一键安装器。

本地ASR唯一接入Qwen3-ASR-1.7B + OpenVINO，默认CPU，需自行按部署文档准备运行库并转换模型；本地摘要另用Qwen3.5-4B GGUF。下载/转换脚本只在开发时主动运行，应用不会自动下载或切换方案。云端仍为默认，用户主动确认保存后才使用本地。

为避免上传个人资料，源码快照将开发机用户名替换为YOUR_USER，将文档里的真实课程标识替换为示例标识，并省略只面向真实课程的回放检查脚本。本机原项目不受这些导出处理影响。测试路径需要迁移；运行前确保使用隔离数据目录。

本地短合成语音及桌面切换验收已通过，真实教室准确率和90分钟稳定性尚未完成。完整实现、当前限制和许可证见其他docs/文档及licenses/。
""",
}
for target, content in extra.items():
    file = OUT / target
    file.parent.mkdir(parents=True, exist_ok=True)
    file.write_text(content, encoding="utf-8")
    raw = file.read_bytes()
    selected[target] = {"path": target, "mode": "100644", "type": "blob", "binary": False, "bytes": len(raw), "git_sha": hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()}

root_readme = OUT / "README.md"
text = root_readme.read_text(encoding="utf-8")
text = text.replace("# ", "# ", 1)
notice = "> GitHub源码快照：此仓库不包含录音、密钥、安装包或大模型。克隆后需准备环境并构建；现有本机项目可继续直接使用。参见 [源码使用说明](docs/GitHub源码使用说明.md)。\n\n"
lines = text.splitlines(keepends=True)
lines.insert(2 if len(lines) > 1 else 1, notice)
root_readme.write_text("".join(lines), encoding="utf-8")
raw = root_readme.read_bytes()
selected["README.md"].update(bytes=len(raw), git_sha=hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest())
manifest = {"root": str(OUT), "files": sorted(selected.values(), key=lambda f: f["path"]), "gitlinks": gitlinks, "excluded_files": excluded}
(ROOT / "build/github-source-manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps({"files": len(selected), "bytes": sum(f["bytes"] for f in selected.values()), "binary": [{"path": f["path"], "bytes": f["bytes"]} for f in selected.values() if f["binary"]], "gitlinks": gitlinks, "excluded_files": excluded}, ensure_ascii=False))
