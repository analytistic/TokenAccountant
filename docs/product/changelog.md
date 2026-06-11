# 产品发布日志

## v0.2 — 审计接入

> 发布日期: 2026-06-11

### 新功能

- **审计记录自动捕获** — 每次代理请求自动记录 input/output token 数，对比上游声称值，可疑请求自动标记
- **请求列表页** — 查看所有审计记录，可疑请求红色高亮，正常灰色展示，一眼识别异常
- **Render Inspector 开发者面板** — 开发者模式下展示 render 前后的文本对比，帮助验证 prefix cache 是否生效
- **SSE 流式转发** — 请求响应逐 chunk 实时转发，不再等全部收完再返回，体验与直连一致

### 改进

- **token 计数更精确** — 消息转换逻辑与 vLLM 推理端完全对齐，确保前端 token 计数与后端一致
- **DeepSeek-V4 支持完善** — 完整支持 DSML 格式的 tool calling、thinking 模式

### 修复

- 修复多轮对话中 store/detect 前缀不匹配导致的 cache 命中率下降
- 修复含图片的请求 token 计数偏低（图片 block 被忽略的问题）
- 修复早期 assistant 的 thinking block 被错误丢弃

---

## v0.1 — MVP

> 发布日期: 2026-06-09

### 新功能

- **Provider 管理** — 添加、切换、删除 AI API provider，支持自定义 API Base URL
- **代理服务器** — 本地 Axum 代理自动改写 Claude settings，零配置对接
- **Token 计数** — 自动统计每次请求的 input/output token 数，支持 DeepSeek 和 GPT 模型
- **审计入库** — 每次请求的 token 用量、差异、可疑原因自动存入 SQLite
