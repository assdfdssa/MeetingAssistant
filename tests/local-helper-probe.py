from pathlib import Path
import json, subprocess, os, time
root=Path(__file__).resolve().parent.parent
model=Path(os.environ['APPDATA'])/'local.classcatch.meetingassistant/models/summary/Qwen3.5-4B-Q4_K_M.gguf'
prompt='<|im_start|>system\n只输出OK，不要解释。<|im_end|>\n<|im_start|>user\n本地模型可用性测试<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n'
request={'type':'generate','prompt':prompt,'model_path':str(model),'max_tokens':32,'context_size':2048,'temperature':0.0,'stop_tokens':['<|im_end|>']}
started=time.monotonic()
with (root/'logs/local-helper-probe-stderr.log').open('w',encoding='utf-8') as err:
    process=subprocess.run([str(root/'build/native/debug/llama-helper.exe')],input=json.dumps(request,ensure_ascii=False)+'\n',stdout=subprocess.PIPE,stderr=err,encoding='utf-8',timeout=180,cwd=root/'build/native/debug')
responses=[]
for line in process.stdout.splitlines():
    try:responses.append(json.loads(line))
    except ValueError:pass
result={'model_path':str(model),'returncode':process.returncode,'seconds':round(time.monotonic()-started,2),'responses':responses,'cloudCallsMade':False}
(root/'logs/local-helper-probe.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(result,ensure_ascii=True))
if process.returncode or not any('OK' in str(r.get('text','')).upper() for r in responses):raise SystemExit(1)
