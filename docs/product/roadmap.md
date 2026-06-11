# TokenAccountant 路线图

## v0.1 (Phase 1) — MVP ✅

Provider 管理 + 基础代理 + token 计数。

| 功能 | 状态 |
|------|------|
| Tauri 2 桌面壳 | ✅ |
| Provider CRUD 管理 | ✅ |
| Axum 代理服务器 | ✅ |
| SSE 流式转发 (初版) | ✅ |
| Token 计数 (tiktoken-rs) | ✅ |
| 请求列表页 | ✅ |

## v0.2 (Phase 2) — 审计接入 ✅

审计接入 + SSE 流式转发 + Render Inspector 开发者面板。

| 功能 | 状态 |
|------|------|
| StreamForwarder SSE 逐 chunk 转发 | ✅ |
| 审计管道 (input/output token 检测 + diff) | ✅ |
| 审计记录 SQLite 存储 | ✅ |
| CacheDetector block hash chain | ✅ |
| message_converter 与 vLLM 格式对齐 | ✅ |
| DeepSeek-V4 DSML 模板匹配官方 | ✅ |
| Render Inspector 开发者面板 | ✅ |
| TemplateParams 参数化 | ✅ |

## v0.3 (Phase 3) — 增强 🔲

计划中，待排期。

| 功能 | 优先级 |
|------|--------|
| 审计趋势图、请求详情弹窗 | 高 |
| Gemini provider 支持 | 中 |
| 缓存跨会话持久化 | 中 |
| Provider 可信度评分 | 低 |
| 熔断 / 故障转移 | 低 |
