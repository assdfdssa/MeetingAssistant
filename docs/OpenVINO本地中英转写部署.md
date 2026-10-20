# OpenVINO 本地中英转写

更新日期：2026-10-19。部署和验收结果以本文末尾及 logs 中对应报告为准。

## 使用方式

设置 → 处理方式 → 检测本地模型（不切换） → 主动选择“纯本地应急（中英混合）” → 勾选确认 → 保存设置。

首次检测需要加载和编译大模型，可能等待几十秒至数分钟。首次CPU验收加载约94秒，之后短语音识别耗时另算。检测不会开启麦克风；只在云端模式做检测后会释放语音模型内存，真正启用本地时可能需要再次加载。

云端 API 仍为默认；下载、检测完成、断网或本地报错均不会自动改变处理方式。录课或摘要生成期间禁止切换。云端密钥保留在 Windows 凭据中，不因本地部署覆盖。

本地转写使用 Qwen3-ASR + OpenVINO，本地摘要仍使用 Qwen3.5-4B GGUF。二者是独立模型。课堂入口不再使用原 Parakeet；上游保留代码和旧模型文件不代表实际启用。

部署采用1.7B模型的INT8权重压缩，以降低内存和磁盘占用；并非把所有运算都变成8位。它仍是1.7B模型。没有与原始BF16模型逐条对照，不能承诺压缩前后准确率完全相同。

## 选定1.7B，停止0.6B路线

用户已明确只使用1.7B。程序只接入1.7B，不提供0.6B入口，不再转换或对照测试0.6B。下表仅解释两版官方差别，不能误读为已做本机双版本跑分。

此前下载到 `models/sources/Qwen3-ASR-0.6B/` 的原始权重未参与当前运行。自动审批拒绝了删除该目录，返回“被策略阻止”，因此文件暂时保留；不将保留文件误标成已删除。

| 项目 | 0.6B | 1.7B |
| --- | --- | --- |
| 标称参数量 | 约6亿 | 约17亿 |
| 官方原始 safetensors 权重 | 1,876,091,704 字节 | 4,698,521,512 字节 |
| 中文会议 WenetSpeech meeting 错误率 | 6.88 | 5.88 |
| 英文 CommonVoice 错误率 | 9.92 | 7.39 |
| 英文 Tedlium 错误率 | 3.85 | 4.50 |
| 取向 | 减少占用和延迟 | 优先准确率，需确认本机速度 |

错误率越低越好。官方表在 BF16 / vLLM 条件下评测，不能当作本机 OpenVINO INT8 的实测值，也不是准确率提高了多少个百分点。1.7B 在多数项目更好，但不是每个样本都更好。原始权重大小不是运行内存，转换后的 INT8 文件大小另行记录。

这台电脑有 Core Ultra X7 358H、Arc B390 核显和约32GB内存。本次只验证1.7B。是否适合实时课堂，需看实际处理速度及与本地摘要同时工作的情况，而不能只看参数量。

## 项目内文件

- `tools/openvino-python/`：可在项目目录运行的 Python、OpenVINO、GenAI、Tokenizer 和 NumPy；使用程序无需再安装系统 Python。
- `tools/openvino-asr.py`：独立识别进程，通过私有标准输入输出管道通信，无网页服务端口。
- `models/qwen3-asr-1.7b-openvino/`：本次唯一接入的本地 OpenVINO IR 语音模型目录。
- `models/summary/Qwen3.5-4B-Q4_K_M.gguf`：已下载摘要模型的项目内副本。
- `models/openvino-cache/`：本机编译缓存；可重新生成，不能代替模型权重。
- `data/local-models.json`：本机所用目录和计算设备。只在重启程序后修改生效，不存 API 密钥。
- `models/sources/`：固定官方提交的原始权重，仅供转换和复现。
- `tools/openvino-export/`：开发时使用的转换环境，包含 PyTorch 等；课堂运行不使用它。
- `logs/openvino-worker.log`：加载或推理异常，不写入音频和正常转写正文。

不要只复制桌面 EXE 到另一台电脑，就认为模型也跟着迁移。当前部署使用项目目录里的运行库及权重；安装包与项目内本地组件的交付范围在 dist/开始使用.md 中说明。

## 工作方式及限制

麦克风采集与录音存储保持原路径。16kHz 单声道音频分段送入持续加载的 OpenVINO 进程，转写写入现有数据库及历史导出路径。摘要生成独立执行，不暂停录音。

当前按约8秒音频段输出最终转写，属于分段实时体验，不是逐字流式显示。时间标记是采集片段时间，不是词级强制对齐。没有额外下载 ForcedAligner。本地术语上下文暂未接入，不将云端热词参数冒充本地已支持功能。

识别队列有上限；跟不上时提示转写可能缺失，原始录音另行保存。本地推理失败会提示并终止该段推理进程，下次请求可重新加载本地模型，不会改用云端。模型和运行库缺失时明确报错，不在课堂运行期间自动下载。

## 可复现验证

`tests/Make-ASR-Samples.ps1` 用 Windows 自带语音生成中文、英文、句内中英混合三份样本，没有使用用户课程或云端 TTS。`tests/openvino-asr.py` 真正调用本地模型，保存文本、处理时长及实时率到 logs。实时率=处理秒数÷音频秒数，小于1代表这份样本处理速度快于播放速度。

合成短样本只能核验接通、关键词和初步速度，不能证明真实教室口音、远场噪声和90分钟稳定性。没有完整测试前不得标注这些项目已通过。

## 来源

- [Qwen3-ASR官方模型及完整评测](https://huggingface.co/Qwen/Qwen3-ASR-1.7B)
- [Qwen3-ASR官方仓库](https://github.com/QwenLM/Qwen3-ASR)
- [OpenVINO GenAI语音识别文档](https://openvinotoolkit.github.io/openvino.genai/docs/use-cases/speech-recognition/)
- [OpenVINO支持模型列表](https://openvinotoolkit.github.io/openvino.genai/docs/supported-models/)

OpenVINO、Qwen3-ASR 使用 Apache-2.0；Python及其依赖保留各自许可。原始来源、提交和权重SHA256见各 sources 目录的 source-manifest.json。

## 本机验收结果

唯一接入模型为 **Qwen3-ASR-1.7B，INT8非对称逐通道权重压缩**。导出模型及相关文件共2,076,294,780字节（约2.08GB），各文件校验值见 `models/qwen3-asr-1.7b-openvino/deployment-manifest.json`。

运行库：OpenVINO 2026.4.1、GenAI 2026.4.1.0、Tokenizer 2026.4.1.0；转换环境为Optimum Intel 2.2.0 / Transformers 4.57.6 / Qwen-ASR 0.0.6。本版本采用 encoder-decoder IR 格式，不与后来文档中的三部分拆分格式混用。

| 本地合成样本 | 音频时长 | CPU识别耗时 | GPU首次该样本耗时 |
| --- | --- | --- | --- |
| 中文：条件概率、下周三提交实验报告 | 7.36秒 | 2.71秒 | 13.77秒 |
| 英文：conditional probability、lab report、Wednesday | 6.86秒 | 1.99秒 | 6.01秒 |
| 句内混合：Bayes theorem、posterior probability | 14.53秒，按约8秒分段 | 合计3.96秒 | 合计4.83秒 |

CPU和GPU均实际完成推理并通过关键词检查。这是短样本首次运行数据，GPU包含初次形状/内核准备开销，不能据此声称所有场景CPU永久更快。本次选择 **CPU、8个推理线程、单路处理**，不在运行时自动变更计算设备。

混合样本中“Bayes theorem”被写成“贝叶斯定理”，而“Posterior probability”保留英文。模型可以输出中英混合内容，但不保证所有英文术语逐字保留。没有用真实教室录音评估词错率，也没有完成90分钟稳定性验收。

Windows桌面隔离数据实测通过：实际加载1.7B语音模型、实际运行Qwen3.5本地摘要测试；检测前后仍为cloud；不确认时拒绝切换；主动确认保存后才为local；本地模式拒绝云端连接测试；最后只将测试副本切回cloud。真实配置保持cloud。检测结束后已确认本地辅助进程退出。

证据：`logs/openvino-asr-1.7B-CPU.json`、`logs/openvino-asr-1.7B-GPU.json`、`logs/native-local-test.json`，界面截图 `build/本地应急-设置实测.png`。原有核心/修复测试10项与课程菜单、专注布局、补听历史/分层界面检查通过。本次不再执行用户那节课程的修复。
