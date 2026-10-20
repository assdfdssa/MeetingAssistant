"""Verify original registry archives, extract without unsupported pre-1980 mtimes.

G: does not accept some archive timestamps. Package bytes/checksums stay unchanged.
No executable package scripts are run by this preparation tool.
"""
import concurrent.futures,hashlib,json,pathlib,tarfile,tomllib,urllib.request,time,sys
root=pathlib.Path(r'G:\MeetingAssistant')
lock_path=pathlib.Path(sys.argv[1]) if len(sys.argv)>1 else root/'meetily/Cargo.lock'
lock=tomllib.loads(lock_path.read_text())
registry=root/'tools/cargo/registry'
cache=registry/'cache/index.crates.io-1949cf8c6b5b557f';cache.mkdir(parents=True,exist_ok=True)
src=registry/'src/index.crates.io-1949cf8c6b5b557f';src.mkdir(parents=True,exist_ok=True)
tarfile.TarFile.utime=lambda *a,**k:None
def prepare(p):
 name,version=p['name'],p['version'];archive=cache/f'{name}-{version}.crate';dest=src/f'{name}-{version}'
 if (dest/'.cargo-ok').exists():return name
 for attempt in range(3):
  try:
   if not archive.exists():
    with urllib.request.urlopen(f'https://static.crates.io/crates/{name}/{name}-{version}.crate',timeout=90) as r: data=r.read()
    if hashlib.sha256(data).hexdigest()!=p['checksum']:raise ValueError(f'Checksum mismatch: {name}')
    archive.write_bytes(data)
   if hashlib.sha256(archive.read_bytes()).hexdigest()!=p['checksum']:raise ValueError(f'Cached checksum mismatch: {name}')
   with tarfile.open(archive,'r:gz') as tar:tar.extractall(src,filter='data')
   (dest/'.cargo-ok').write_text('{"v":1}')
   return name
  except Exception:
   if attempt==2:raise
   time.sleep(2)
packages=[p for p in lock['package'] if p.get('source','').startswith('registry+')]
errors=[]
with concurrent.futures.ThreadPoolExecutor(max_workers=12) as pool:
 for i,job in enumerate(concurrent.futures.as_completed([pool.submit(prepare,p) for p in packages]),1):
  try:job.result()
  except Exception as e:errors.append(str(e));print('ERROR',e,flush=True)
  if i%50==0:print(f'Prepared {i}/{len(packages)} verified packages',flush=True)
(root/'logs'/('cargo-cache-core.json' if len(sys.argv)>1 else 'cargo-cache.json')).write_text(json.dumps({'packages':len(packages),'errors':errors},indent=2))
if errors:raise SystemExit(1)
