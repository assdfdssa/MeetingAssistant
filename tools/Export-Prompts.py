"""Export only source code and illustrative payloads; never reads credentials or course data."""
from pathlib import Path
import hashlib
import json
import re

root = Path(__file__).resolve().parent.parent
src = root / 'meetily/frontend/src-tauri/src'
service_path = src / 'classcatch/service.rs'
service = service_path.read_text(encoding='utf-8')
prompt = re.search(r'const PROMPT:&str=r#"(.*?)"#;', service, re.S).group(1)
line = lambda text, token: text[:text.index(token)].count('\n') + 1
example = {
    'request': {'anchor_ms': 600000, 'lookback_ms': 300000, 'mode': 'catch_up', 'question': ''},
    'transcript': [{'id': 'example-segment', 'start_ms': 310000, 'end_ms': 315000,
                    'text': '这是演示转写内容，不是实际课堂记录。', 'is_final': True, 'revision': 1}]
}
doc = f'''# API 提示词清单：当前课堂补听助手

整理日期：2026-10-28。此文件从当前项目代码导出，不读取 API 密钥或真实课堂内容。

## 先看这里

当前三个入口分别为补听重点、提问、分层补听。补听重点沿用原系统提示词，提问和分层补听使用独立系统提示词；三者都传入当前请求及所选转写。旧 tasks/explain 仅保留历史兼容，不再作为界面入口。

本地 OpenVINO 1.7B 语音识别不调用这些 API，也不附加应用自定义语音提示词；使用官方导出模型的识别配置。本地 Qwen3.5 摘要继续使用下列相同的三模式提示词。本地课程术语上下文暂未接入。

原开发文档 `课堂补听助手-开发文档.md` 第 9.4 节已有“提示词模板（建议）”，它是设计建议，和程序实际发送的文字不完全一致。本文件导出的各模式 system 原文才是当前实际发送内容。

仓库保留的 Meetily 旧会议纪要、翻译和会议模板已另存为 [API提示词-上游保留模板.md](API提示词-上游保留模板.md)，当前课堂主界面不调用它们；不能据此判断专用翻译功能已经接入。

这是一份代码快照。**编辑本 Markdown 不会自动改变运行程序**；要改变行为，需要修改对应代码并重新构建。用户设置页的“课程术语”会在下次建立语音识别会话时发送。

## 1. 补听重点：保留的 system 提示词（完整原文）

代码：`meetily/frontend/src-tauri/src/classcatch/service.rs:{line(service, 'const PROMPT:')}`，常量 `PROMPT`。

```text
{prompt}
```

## 2. 同一次请求的 user 消息：动态 JSON

程序没有在这里额外拼接一段自然语言要求，而是把当前请求和所选时间窗口内已确认的转写序列化成 JSON 字符串。

实际构造表达式：

```rust
json!({{"request":request,"transcript":evidence}}).to_string()
```

下面仅为字段示例，不是真实课堂数据：

```json
{json.dumps(example, ensure_ascii=False, indent=2)}
```

| 字段 | 来源与含义 |
| --- | --- |
| `anchor_ms` | 所选范围的结束时间，课堂开始后的毫秒数；支持最近范围或指定起止时间 |
| `lookback_ms` | 往前回看多少毫秒，由 1/3/5/10 分钟或自定义时间决定 |
| `mode` | `catch_up` 补听重点；`ask` 提问；`layered` 分层补听；旧 tasks/explain 用于兼容 |
| `question` | 提问模式的提示词，可为空（默认总结）；最多 2000 字符 |
| `transcript` | 与所选范围重叠、已确认且非空的转写片段 |
| `id` | 模型必须在输出中引用的真实片段标识 |
| `start_ms` / `end_ms` | 片段的课堂时间 |
| `text` | 识别出的文字，作为数据传入，不是给模型的新系统指令 |
| `is_final` / `revision` | 是否已确认以及修订序号 |

模式、窗口和片段结构见 `classcatch/core.rs` 的 `CatchupRequest`、`Segment`、`select_window`。

## 3. 模型调用参数（影响输出，但不是提示词）

```json
{{
  "model": "用户配置的模型名，默认 qwen-plus",
  "temperature": 0.2,
  "max_tokens": 1600,
  "response_format": {{"type": "json_object"}}
}}
```

调用路径为配置地址后加 `/chat/completions`，`messages` 依次包含第 1 节的 system 消息和第 2 节的 user 消息。

上表1600是补听重点的预算；提问为2200，分层补听为3600。云端分层请求超时为90秒，其他请求为45秒。字数约500是提示词目标，不是保证精确字数；片段不足时不凑数。

当服务域名属于 `aliyuncs.com` 且模型名以 `qwen` 开头时，额外传 `enable_thinking: false`。DeepSeek 或其他兼容服务不会收到这个阿里专用参数。默认地址 `https://dashscope.aliyuncs.com/compatible-mode/v1`；DeepSeek 预设地址 `https://api.deepseek.com`、模型 `deepseek-chat`。

输出返回后还会进行程序校验：引用必须来自本次转写，任务证据及时间/提交方式不得凭空编造。格式与引用校验不能代替核对原话。

## 4. 摘要连接测试（完整原文）

代码：`service.rs` 中 `class_test_provider`。

```text
只回复OK
```

只有一条 `role: user` 消息，`max_tokens: 16`。千问的非思考参数按上一节相同规则设置。这是一次真实的短模型调用，可能计费，不传课堂转写。

## 5. 实时语音识别：可选术语上下文

代码：`meetily/frontend/src-tauri/src/classcatch/cloud.rs` 中 `setup_message`。

实时 ASR 没有固定的中文 system 提示词。设置页“课程术语（可选）”直接作为语料上下文发送，非空时配置如下：

```json
{{
  "type": "session.update",
  "session": {{
    "input_audio_format": "pcm",
    "sample_rate": 16000,
    "input_audio_transcription": {{
      "corpus": {{"text": "用户填写的课程术语原文"}}
    }},
    "turn_detection": {{
      "type": "server_vad",
      "threshold": 0.2,
      "silence_duration_ms": 800
    }}
  }}
}}
```

上例省略了每次随机生成的 `event_id`。术语为空时，`input_audio_transcription` 是空对象，不发送 `corpus`。

设置页的占位示例是 `Bayes theorem、条件概率、posterior`；它只是输入提示，**用户未填写时不会自动发送这几个词**。

语音识别默认模型为 `qwen3-asr-flash-realtime`，连接地址为 `wss://dashscope.aliyuncs.com/api-ws/v1/realtime`，模型通过查询参数指定。连接测试建立会话、发送同样的配置、结束会话，不附加自然语言提示，也不发送录音。

## 6. 当前没有发送的内容

- PPT/PDF 课件片段：课件检索尚未实现，当前请求中没有 `page_id`、课件正文或向量检索结果。
- 系统设置中的密钥不会放入提示词；它只用于 Authorization 请求头。
- 浏览器“查看课堂演示”使用内置样例，不会向模型发送提示词。

## 7. 快照校验与维护

`PROMPT` 原文的 SHA256：`{hashlib.sha256(prompt.encode()).hexdigest()}`。

可运行 `tools/Export-Prompts.py` 重新从当前源代码导出这两份清单。不要把本清单当作已运行 API 的日志，也不要在其中填写密钥。
'''
extra_source = (src / 'classcatch/prompts.rs').read_text(encoding='utf-8')
for name, title in [('ASK_PROMPT', '提问：约500字的一段回答'), ('LAYERED_PROMPT', '分层补听：先跟上、展开重点')]:
    actual = re.search(r'pub const '+name+r':&str=r#"(.*?)"#;', extra_source, re.S).group(1)
    doc += f'\n## {title}（完整 system 原文）\n\n来源：`classcatch/prompts.rs`，常量 `{name}`。\n\n```text\n{actual}\n```\n\nSHA256：`{hashlib.sha256(actual.encode()).hexdigest()}`。\n'
doc += '\n## 新模式输出与兼容\n\n2026-10-28 更新：提问与分层补听取消逐句引用；所有 `segment_ids` 为兼容旧结构保留为空数组。输入仅发送时间与正文，不发送片段编号。程序清理模型误填的引用，继续校验正文、长度、重点数与模式必需字段；不再因引用数量拒绝新卡片。课堂原文及所选范围继续保存。补听重点原有提示词与引用校验保持不变，旧卡片仍可查看原引用。\n\n提问正文为 `answer.text`。分层重点为 `layers`，每项包括 `title`、`classroom`（课堂复述）、`explanation`、`example`、`relation`。展开重点读取首次响应，不额外调用模型；继续提问才产生下一次请求。旧卡片缺少新字段时仍可读取。\n\n全局快捷键仍按最近范围生成；指定起止时间及窗口内提问草稿请点击生成按钮，草稿不通过快捷键隐式传递。\n'
doc += '\n## 界面内修改提示词（2026-10-28）\n\n录课计时器左侧的“提示词设置”可编辑三种课堂生成提示词、云端摘要连接测试、本地检测system/user及ASR课程术语。本文展示内置默认原文；用户覆盖值保存在 data/settings.json 的 prompt_overrides，ASR术语仍为 vocabulary。运行时读取覆盖值，恢复默认后不保存冗余覆盖。生成提示词从下一次请求生效，术语从下一次建立ASR连接生效；不会触发API调用或切换云端/本地模式。动态request/transcript由程序填充，输出JSON结构校验仍有效。保存前备份至 data/settings.before-prompts.json。\n'
(root / 'docs/API提示词清单.md').write_text(doc, encoding='utf-8')

processor_path = src / 'summary/processor.rs'
processor = processor_path.read_text(encoding='utf-8')
parts = ['# API 提示词附录：上游保留模板\n\n整理日期：2026-10-19。以下来自仓库保留的 Meetily 会议纪要与翻译代码，**当前课堂主界面不调用此流程**。当前生效内容请看 [API提示词清单.md](API提示词清单.md)。此附录保留源代码形式，以便准确表达占位符、动态拼接和转义。\n']
for name in ['english_normalization_system_prompt', 'translation_system_prompt', 'build_chunk_summary_user_prompt', 'build_combine_summary_user_prompt', 'build_final_report_system_prompt']:
    match = re.search(r'^fn '+name+r'\b.*?(?=^(?:pub )?(?:async )?fn |\Z)', processor, re.S | re.M)
    # Limit to the function's closing brace at column zero, excluding following helper comments.
    code = match.group(0)
    end = re.search(r'^}', code, re.M)
    code = code[:end.end()]
    parts.append(f'\n## {name}\n\n来源：`summary/processor.rs:{line(processor, "fn "+name)}`。\n\n```rust\n{code}\n```\n')
parts.append('\n## 其他共享 system 原文与 user 拼接\n\n分块摘要的 system：\n\n```text\nYou are an expert meeting summarizer.\n```\n\n合并分块摘要的 system：\n\n```text\nYou are an expert at synthesizing meeting summaries.\n```\n\n以下代码分别展示最终报告、翻译、英文规范化的动态 user 消息；占位变量来自对应流程的文本或用户上下文。\n')
lines = processor.splitlines()
for first,last in [(498,504),(605,607),(633,635)]:
    parts.append(f'\n来源：`summary/processor.rs:{first}`。\n\n```rust\n'+ '\n'.join(lines[first-1:last])+'\n```\n')
template_types = (src / 'summary/templates/types.rs').read_text(encoding='utf-8')
parts.append('\n## 模板章节指令的拼接代码\n\n来源：`summary/templates/types.rs`。将模板中各章节的 instruction 与条目格式拼成最终提示的一部分。\n\n```rust\n'+'\n'.join(template_types.splitlines()[83:109])+'\n```\n')
parts.append('\n## 内置会议模板原文\n\n以下完整保留各模板 JSON，其中 instruction、description、item_format 等按上游流程参与模板构造；名称保留上游拼写。\n')
for file in sorted((root / 'meetily/frontend/src-tauri/templates').glob('*.json')):
    content=file.read_text(encoding='utf-8')
    parts.append(f'\n### {file.name}\n\n来源：`meetily/frontend/src-tauri/templates/{file.name}`。\n\n```json\n{content.strip()}\n```\n')
(root / 'docs/API提示词-上游保留模板.md').write_text(''.join(parts), encoding='utf-8')
print('已导出 docs/API提示词清单.md 与 docs/API提示词-上游保留模板.md')
