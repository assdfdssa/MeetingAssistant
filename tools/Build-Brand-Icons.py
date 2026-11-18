"""Build project-native SVG artwork into the desktop and browser icon assets."""
from pathlib import Path
import re
import shutil
import subprocess
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
FRONTEND = ROOT / 'meetily/frontend'
ICONS = FRONTEND / 'src-tauri/icons'
OUTPUT = ROOT / 'build/brand-icons'
node = ROOT / 'tools/node/node.exe'
subprocess.run([str(node) if node.exists() else 'node', 'node_modules/@tauri-apps/cli/tauri.js', 'icon',
                'src-tauri/icons/classcatch.svg', '--output', str(OUTPUT)], cwd=FRONTEND, check=True)
for file in OUTPUT.iterdir():
    if file.is_file() and file.suffix in {'.png', '.ico', '.icns'}:
        shutil.copy2(file, ICONS / file.name)
with Image.open(OUTPUT / 'icon.png') as original:
    original = original.convert('RGBA')
    for file in ICONS.glob('icon_*x*.png'):
        match = re.fullmatch(r'icon_(\d+)x(\d+)(@2x)?\.png', file.name)
        if match:
            scale = 2 if match[3] else 1
            original.resize((int(match[1])*scale, int(match[2])*scale), Image.Resampling.LANCZOS).save(file)
shutil.copy2(ICONS / 'icon.ico', ICONS / 'app_icon.ico')
shutil.copy2(ICONS / 'icon.icns', ICONS / 'app_icon.icns')
shutil.copy2(ICONS / 'icon.ico', FRONTEND / 'src/app/favicon.ico')
shutil.copy2(ICONS / 'classcatch.svg', FRONTEND / 'public/classcatch-mark.svg')
with Image.open(ICONS / 'icon.ico') as ico:
    sizes = sorted(ico.info.get('sizes', []))
    if not {(16,16),(32,32),(48,48),(256,256)}.issubset(set(sizes)):
        raise RuntimeError(f'Missing Windows icon sizes: {sizes}')
print(f'Brand icons generated and verified: {sizes}')
