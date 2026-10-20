# GitHub源码快照

此仓库保存课堂补听助手源码、文档、脚本、测试和第三方许可。Meetily源码放在meetily/，保留其上游说明和MIT许可。本项目基于Meetily Community v0.4.1，上游提交a2cb62e827da7ef59f65064c97233efb2313878e。

录音、转写数据库、API密钥、诊断日志、安装包、本地模型、Python/Node/Rust工具链和构建缓存不在仓库里。dist/只有交付说明和构建校验记录，没有可直接下载的安装包。源代码拉取后不能直接双击启动脚本运行，必须准备开发环境并构建。

推荐拉取命令：git clone --recurse-submodules https://github.com/assdfdssa/MeetingAssistant.git 。根目录.gitmodules固定whisper.cpp子模块关系。

Windows开发需要Node/pnpm、Rust、MSVC C++与Windows SDK、CMake/Ninja/Clang等，并准备Tauri所需FFmpeg、ONNX Runtime与release版llama-helper。tools/Build-Environment.ps1和部分测试脚本对应原开发机目录，迁移时请按实际目录修改，不把已验证的本机环境误当作通用一键安装器。

本地ASR唯一接入Qwen3-ASR-1.7B + OpenVINO，默认CPU，需自行按部署文档准备运行库并转换模型；本地摘要另用Qwen3.5-4B GGUF。下载/转换脚本只在开发时主动运行，应用不会自动下载或切换方案。云端仍为默认，用户主动确认保存后才使用本地。

为避免上传个人资料，源码快照将开发机用户名替换为YOUR_USER，将文档里的真实课程标识替换为示例标识，并省略只面向真实课程的回放检查脚本。本机原项目不受这些导出处理影响。测试路径需要迁移；运行前确保使用隔离数据目录。

本地短合成语音及桌面切换验收已通过，真实教室准确率和90分钟稳定性尚未完成。完整实现、当前限制和许可证见其他docs/文档及licenses/。
