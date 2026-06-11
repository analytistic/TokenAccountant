# Changelog

## [0.2.0] - 2026-06-11

### Added
- Render Inspector 开发者面板（DevTraceBuffer + IPC + DevPanel 组件 + 开发者模式开关）
- DevPanel 深色主题、Store/Detect 左右并排、请求列表底部条
- TemplateParams 参数化（thinking_mode / reasoning_effort / drop_thinking / add_default_bos_token）
- extract_template_params() 从请求体提取渲染参数
- Tool call per-parameter DSML 编码（匹配官方 deepseek_v4_encoding）
- sort_tool_results_by_call_order（按 tool_call 顺序排序 tool_result）
- reasoning_effort='max' 前缀支持
- Developer / response_format / wo_eos 支持
- Tool_reference 支持

### Changed
- message_converter: ContentPart 枚举替代 flat content string，支持 image / text / tool_reference
- NormalizedMessage: content_parts 替代 content，新增 tool_call_id 字段
- deepseek tokenizer: 完全重写，匹配 vLLM deepseek_v4_encoding.py
- merge_tool_messages: content_blocks 格式替代字符串拼接
- _drop_thinking_messages: 匹配官方逻辑
- store 路径保留 assistant reasoning（去掉 store_msg.reasoning = None）
- Tokenizer trait: 新增 apply_chat_template_with 方法

### Fixed
- store/detect 前缀不匹配（reasoning 被剥离导致）
- 多个 system 消息只处理第一个
- image block 被静默丢弃（转为 ContentPart::ImageUrl data URI）
- redacted_thinking 透传问题
- tool_result 子结构缺失（text + image + tool_reference）
- 用户消息空内容时仍 push（匹配 vLLM "content" not in openai_msg 守卫）
- tool 消息始终创建即使内容为空

## [0.1.0] - 2026-06-09

### Added
- Tauri 2 桌面壳 + React 前端
- Provider CRUD 管理
- Axum 代理服务器
- SSE 流式转发
- Token 计数（tiktoken-rs + claude fallback）
- CacheDetector block hash chain（LCG 确定性 hash）
- 审计记录 SQLite 存储
- DeepSeek DSML 渲染（transition tokens / think tags / EOS）
- x-anthropic-billing-header 过滤（cache-busting）
- 配置集中化 AppConfig
