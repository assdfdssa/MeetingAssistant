# 课堂补听助手：主底座二次深度检索与选型

> 最终决定（2026-10-15）：用户已确认 **Meetily Community Edition为唯一主底座**。本文保留检索阶段的 Star 快照与候选比较作为决策依据；其他项目只作参考，不再安排三底座对比开发。当前架构以[主开发文档](./课堂补听助手-开发文档.md)与[Meetily实施清单](./Meetily实施映射与迁移清单.md)为准。后续首版开发已完成基础验证，实际进度见[开发进度与验证](./开发进度与验证.md)；下文检索阶段的“未下载安装”不包含后续开发测试。

日期：2026-10-15（北京时间）。Star为本轮GitHub API读取快照，之后会变动；这是候选集的排序，不是GitHub全部相关项目的全球排行榜。核查22个仓库元数据，重点读取README、许可、发布资产与部分关键源码；未下载安装，不能据此保证稳定性。最近推送时间不是最近一次实质功能更新的证明。

## 结论与对原建议的修正

检索阶段曾将Meetily、Buzz、NexQ列为不同取向的候选。现已由用户确定 **Meetily为主底座**，不再保留“先对比再决定”的执行安排。Buzz的Python路线、NexQ的交互和MeetingBro的原型设计只作研究记录，不能作为并行主架构。Meetily仍需接入云端ASR并开发录制中按需补听，选定不等于开箱满足全部需求。

“更好”取决于缺少哪一部分：可靠的录课与安装发行、适合自己读懂的代码、还是快捷键加可选时间窗口。不能仅凭Star确认云端识别接入难度或两小时稳定性。

## 按Star降序的候选集

| 排名 | 仓库 | Star | 最近推送（UTC） | 判断 |
|---|---|---:|---|---|
| 1 | [Zackriya-Solutions/meetily](https://github.com/Zackriya-Solutions/meetily) | 31,751 | 2026-09-15 | 已确定为唯一主底座；Rust/Tauri，国内流式ASR与补听需改造 |
| 2 | [chidiwilliams/buzz](https://github.com/chidiwilliams/buzz) | 21,998 | 2026-10-11 | Python主导，适合自主修改；现有AI摘要为完成后钩子，非课堂即时补听 |
| 3 | [screenpipe/screenpipe](https://github.com/screenpipe/screenpipe) | 21,944 | 2026-10-15 | 最新源码为商业许可；范围偏全电脑记录，不选作默认底座 |
| 4 | [QuentinFuxa/WhisperLiveKit](https://github.com/QuentinFuxa/WhisperLiveKit) | 11,143 | 2026-10-01 | 实时语音服务组件，不是完整课堂桌面应用 |
| 5 | [fastrepl/anarlog](https://github.com/fastrepl/anarlog) | 9,513 | 2026-10-14 | 原Hyprnote；社区MIT、enterprise商业；Windows beta，仓库较大 |
| 6 | [thewh1teagle/vibe](https://github.com/thewh1teagle/vibe) | 7,781 | 2026-10-06 | 跨平台转写应用与HTTP API；录制中整段补听需另外确认/开发 |
| 7 | [SevaSk/ecoute](https://github.com/SevaSk/ecoute) | 6,039 | 2026-04-08 | Python实时转写参考；默认设备及非API英语限制，产品层薄 |
| 8 | [sohzm/cheating-daddy](https://github.com/sohzm/cheating-daddy) | 5,628 | 2026-07-23 | Electron实时浮层；偏Gemini音视频问答，非本地录课档案 |
| 9 | [matthartman/ghost-pepper](https://github.com/matthartman/ghost-pepper) | 3,202 | 2026-07-27 | macOS项目，排除Windows主底座；许可证未完成核实 |
| 10 | [pluja/whishper](https://github.com/pluja/whishper) | 3,089 | 2026-07-31 | Web转写/字幕工具，AGPL；不是首选桌面实时课堂路径 |
| 11 | [Natively-AI-assistant/natively-cluely-ai-assistant](https://github.com/Natively-AI-assistant/natively-cluely-ai-assistant) | 2,972 | 2026-10-15 | 当前个人使用源码许可，非通用开源授权，未来商业路线受限 |
| 12 | [Vexa-ai/vexa](https://github.com/Vexa-ai/vexa) | 2,885 | 2026-10-11 | 在线会议机器人/API，不能替代线下麦克风桌面前端 |
| 13 | [iamsrikanthnani/pluely](https://github.com/iamsrikanthnani/pluely) | 2,742 | 2026-09-21 | 最新v1闭源；旧GPL代码仍在历史，当前产品功能不可归给旧源码 |
| 14 | [Prat011/free-cluely](https://github.com/Prat011/free-cluely) | 1,691 | 2026-04-07 | Electron浮层参考；README许可文字与LICENSE不一致，需以具体版本核实 |
| 15 | [amicalhq/amical](https://github.com/amicalhq/amical) | 1,557 | 2026-10-13 | 听写/快捷操作参考；README把实时会议转写标为计划中 |
| 16 | [silverstein/minutes](https://github.com/silverstein/minutes) | 1,550 | 2026-10-15 | 本地会议记忆/MCP；桌面路径偏Mac，不作为Windows优先候选 |
| 17 | [Muesli-HQ/muesli](https://github.com/Muesli-HQ/muesli) | 1,414 | 2026-10-14 | 原生macOS/Swift，排除Windows主底座 |
| 18 | [Valtora/Nojoin](https://github.com/Valtora/Nojoin) | 83 | 2026-10-15 | 元数据候选；许可与桌面适配未深核，不给优先推荐 |
| 19 | [naxhq/NexQ](https://github.com/naxhq/NexQ) | 65 | 2026-09-28 | 功能吻合度高：Windows浮层、窗口转写、任务/RAG；低Star，需长时实测 |
| 20 | [armpro24-blip/MeetingBro](https://github.com/armpro24-blip/MeetingBro) | 43 | 2026-06-23 | Python/Electron改造原型；滚动摘要实际只取新增片段；无GitHub Releases |
| 21 | [estebanstifli/Meet2Notes](https://github.com/estebanstifli/Meet2Notes) | 30 | 2026-10-01 | Python模块化备选；规模小，需验证安装及长时流式表现 |
| 22 | [talberthoule/backchannel](https://github.com/talberthoule/backchannel) | 9 | 2026-09-25 | 实时转写/智能体备选；规模很小，非主候选 |

Star直接来自对应仓库的GitHub API stargazers_count。仓库显示的主要语言不等于只有一种开发语言；混合许可不能仅依赖API license字段。

## 1. Meetily：已确定的主底座

- Star 31,751。已核实v0.4.1于2026-09-12发布，包含Windows exe与msi；此前也有v0.4.0、v0.3.0。
- 已有麦克风/系统音频、实时转写、历史、本地保存、摘要提供商设置；核心为Rust/Tauri，界面为Next.js/React。仓库存在旧Python后端内容，不能据此认为当前主线以Python为核心。
- 读到的录音实现包含停止收尾状态、设备故障处理、转写事件持久化和并行任务，这些说明已有工程处理，但没有替代真实稳定性测试。
- 关键改造：阿里云WebSocket ASR生命周期、停用本地模型下载/启动前校验、按键锚点范围选择、录制中异步摘要、简洁卡片。
- 不能只配置“自定义OpenAI端点”就推断阿里流式ASR能使用；它首先是文本摘要能力，不代表语音协议兼容。
- 不能把PRO功能算成社区代码已经具有；社区版与PRO是不同代码路线。
- 已决定沿用Rust/Tauri与Next.js/React，后续通过模块解释与小步实施降低学习负担，不转写成Python主程序。

证据：[README](https://github.com/Zackriya-Solutions/meetily/blob/main/README.md)、[发行版](https://github.com/Zackriya-Solutions/meetily/releases/tag/v0.4.1)、[录音命令层](https://github.com/Zackriya-Solutions/meetily/blob/a2cb62e827da7ef59f65064c97233efb2313878e/frontend/src-tauri/src/audio/recording_commands.rs)。

## 2. Buzz：最值得补充进原方案的Python候选

- Star 21,998。README有Windows安装指引，GitHub v1.4.5包含Windows安装资产。
- Python/PyQt6，有麦克风实时转写、导出、回放/浏览和插件机制。比另学Rust更符合你的已有基础，但界面和交互仍需开发，Python不等于无需理解GUI线程与打包。
- 源码录音识别支持自定义OpenAI音频接口，但看到的是音频分片写WAV后调用audio.transcriptions.create，不能当成阿里实时WebSocket协议的现成适配。
- 已有AI Summary插件使用on_complete，文档说明转写完成后运行；不能只开插件就达到录制中即时补听。
- 录音线程、时间戳、长期原始音频保存、实时片段暴露与全局快捷键，需要独立核查/改造。尤其当前部分本地识别分支关闭词级时间戳，不能直接承诺精确到词的范围。
- 许可证：仓库MIT；依赖PyQt6。PyQt为GPLv3/商业双许可，不是LGPL。以后拟闭源分发应处理该依赖许可；不代表Buzz不能商用，也不影响其作为个人学习候选的价值。
- 默认分支pyproject与README的Python版本文字存在差异；使用固定发行标签及锁定依赖，不能拼装不同版本的安装说明。

证据：[README](https://github.com/chidiwilliams/buzz/blob/main/README.md)、[录音识别](https://github.com/chidiwilliams/buzz/blob/851c8b359343db8c9460aa15ee59a40f2ab66aa8/buzz/transcriber/recording_transcriber.py)、[插件说明](https://github.com/chidiwilliams/buzz/blob/main/docs/docs/usage/7_plugins.md)、[AI摘要完成钩子](https://github.com/chidiwilliams/buzz/blob/851c8b359343db8c9460aa15ee59a40f2ab66aa8/buzz/plugins/ai_summary/plugin.py#L96)、[PyQt官方许可](https://en.riverbankcomputing.com/software/pyqt)。

## 3. Anarlog（原Hyprnote）：产品框架完整，但不默认更省事

- Star 9,513；已核实desktop_v1.4.30含Windows安装包。
- 社区应用MIT，enterprise目录商业许可；需按路径选择，不可把整个单体仓库一概当MIT。
- 有录音、本地SQLite、转写与摘要分离、BYOK、历史和多应用结构。适合参考产品体验与提供商抽象。
- Windows仍有beta平台说明；内置Apple Speech/Soniqo等Mac能力不能归给Windows版本。
- Rust/Tauri加多包工作区，网站、可选服务、同步和移动端内容较多。个人课堂小工具可能需要大量裁剪；功能丰富也会增加编译和维护负担。
- 默认建议排第三梯队验证，而不是因为UI成熟就立即替换底座。

证据：[README及命名历史](https://github.com/fastrepl/anarlog/blob/main/README.md)、[许可边界](https://github.com/fastrepl/anarlog/blob/main/LICENSING.md)、[平台说明所在官方仓库](https://github.com/fastrepl/anarlog/blob/93b5e38c21e61dd1e17d609b7c965a435f8d7543/apps/web/content/articles/free-transcription-software-windows.mdx)、[Windows发行](https://github.com/fastrepl/anarlog/releases/tag/desktop_v1.4.30)。

## 4. NexQ：最接近你的产品流程的小项目

- Star 65；不是高Star成熟性推荐。已核实v2.21.1有Windows安装包。
- README列出麦克风/系统音频、浮层、实时辅助、任务、翻译、资料检索、WAV与历史回放。Rust/Tauri/React，不是Python路线。
- 源码generateAssist取当前已确认转写，传到后台；后台按每种动作的transcript_window_seconds选范围，并异步调用模型，说明并非只在README里宣称有实时辅助。
- 已找到全局快捷键实现，资料检索与音频回放同步入口。仍未验证这些功能在Windows连续两小时使用是否稳定。
- 重要缺口：窗口终点取“最后一条转写timestamp”，不是用户按键时间。老师静音、ASR延迟时会选择错范围。需要改为按键锚点，加水位与缺口提示。
- 部分生成路径只携带文本/说话人/时间，完整证据ID与版本链需补齐；不能把有时间窗口等同于可靠可追溯。
- README“100% Local”与云端可选服务要分开理解，使用云端ASR/LLM时相关数据仍会外发。
- 阿里实时ASR和DeepSeek/Qwen接入需要实际确认，代码里出现模型名称不代表供应商端到端兼容。

证据：[功能与技术栈](https://github.com/naxhq/NexQ/blob/main/README.md)、[前端生成入口](https://github.com/naxhq/NexQ/blob/c2a6d33328d4bc6f3d3186eba4a8ffcea3665cda/src/lib/ipc.ts#L192)、[时间窗口实现](https://github.com/naxhq/NexQ/blob/c2a6d33328d4bc6f3d3186eba4a8ffcea3665cda/src-tauri/src/commands/intelligence_commands.rs#L49)、[快捷键](https://github.com/naxhq/NexQ/blob/main/src/hooks/useGlobalShortcut.ts)、[Windows发行](https://github.com/naxhq/NexQ/releases/tag/v2.21.1)。

## 5. 为什么一些高Star项目不作主底座

- **WhisperLiveKit**：很好的实时语音服务底层，有API与翻译；缺少完整课堂桌面体验。适合作组件或本地后备，不与Meetily的完整应用定位等同。
- **Screenpipe**：当前LICENSE.md为商业许可，允许部分非商业用途，但商业/竞争产品有限制；旧MIT版本需单独固定验证。最新产品偏全电脑工作记录，超出无需视觉的课堂工具。
- **Natively**：当前LICENSE明确source-available，个人、教育与非商业使用路线；未来商业派生需要另行许可。不能依据仓库介绍中的open-source关键词判断可闭源商用。
- **Pluely**：当前README明确v1闭源；历史GPL版本与当前产品不是同一份可fork代码。API返回GPL字段不能覆盖当前版本事实。
- **Amical**：浮动窗口与快捷操作值得学习，但README把实时会议转写列作计划中。听写成熟不代表长时录课成熟。
- **Ecoute**：Python实现易于阅读，但文档明确默认音频设备限制、非API路径英语限制；要变成可靠课堂程序仍需补足存储、时间轴、交互和安装。
- **Vexa**：侧重网课/会议机器人与API，与你线下外置麦克风为主的输入方式不匹配。
- **Vibe**：有跨平台录音转写、摘要和HTTP API，是第二梯队备选；README的realtime preview不足以证明录制中按需窗口摘要已实现。
- **Muesli/Ghost Pepper/Minutes**：平台或主工作流偏macOS，不作为Windows第一版主底座。Ghost Pepper仅完成元数据筛查，不给许可肯定判断。
- **Free Cluely/Cheating Daddy**：偏浮层问答与音视频分析；长期音频归档、稳定课堂时间范围仍需核验。Free Cluely README写ISC，但实际LICENSE为Apache-2.0，说明不能只看介绍文字。

事实依据：[Screenpipe许可](https://github.com/screenpipe/screenpipe/blob/main/LICENSE.md)、[Natively许可](https://github.com/Natively-AI-assistant/natively-cluely-ai-assistant/blob/main/LICENSE)、[Pluely当前说明](https://github.com/iamsrikanthnani/pluely/blob/master/README.md)、[Amical功能状态](https://github.com/amicalhq/amical/blob/main/README.md)、[Ecoute限制](https://github.com/SevaSk/ecoute/blob/main/README.md)、[Free Cluely许可](https://github.com/Prat011/free-cluely/blob/master/LICENSE)。

## 6. 对你具体怎样选

| 你优先解决的问题 | 推荐验证对象 | 需要接受的代价 |
|---|---|---|
| 我主要想拿到体验完整、能给同学安装的程序 | Meetily | Rust学习/辅助开发，云端ASR与按需补听仍需开发 |
| 我希望自己读懂代码、逐步改出来 | Buzz | PyQt界面与实时补听需要开发，未来发行看依赖许可 |
| 我想尽量少重做浮层、资料检索、任务交互 | NexQ | 社区小、Rust，需严格验证时间轴和连续录课 |
| 我更重视完整笔记产品体系 | Anarlog | 仓库较复杂，Windows beta，裁剪成本 |
| 我只需要Python+Electron的小原型 | MeetingBro | 无Releases，需自建安装与可靠性验收，摘要逻辑需修正 |

上表保留的是研究阶段的取舍，不是当前执行计划。当前只固定Meetily社区版基线（建议v0.4.1并记录commit SHA），按以下顺序实施验证：

1. Windows选择外置麦克风，录音转写20分钟并回放。
2. 录制中按键，读取最近1/5/10分钟完整转写并单独调用一次模型，录音继续。
3. 在Meetily接入阿里流式ASR，调整本地模型启动门槛，统一转写、保存与历史路径。
4. 用“下周不用交”“周五改周日”等样例验证任务提取及证据回放。
5. 两节50分钟加课间、至少120分钟稳定性和安装包测试。

底座已由用户选定；以上验证用于实施与验收，不作为重新选择底座的前提。遇到问题先在Meetily内定位和解决；若确实需要改变底座，另行说明具体阻碍，不能自动恢复为多候选架构。当前没有安装实测结果，不把选型决定写成性能保证。

## 7. 与原开发文档的关系

主开发文档已升级v1.1，架构统一为Meetily社区版的Tauri/Rust与Next.js/React；内部命令使用Tauri IPC，不增设Python服务或第二个桌面壳。产品目标、证据规则、时间轴、持续录课、课件增强和验收标准保留。Star、检索日期和其他仓库事实仍是原始研究快照，不因选型改变而伪造更新。

