# vLLM Anthropic → OpenAI 转换器

**来源:** vLLM 仓库 `vllm/entrypoints/anthropic/serving.py`
**用途:** Phase 2 审计接入调研参考 — 展示 vLLM 如何将 Anthropic API 格式转换为 OpenAI ChatCompletionRequest

## 关键方法

### `_convert_anthropic_to_openai_request()`
将 Anthropic 请求转为 OpenAI 格式：
1. `_convert_system_message` — 顶层 `system` + msg role=system → 一条 `{role:"system", content:"..."}`
2. `_convert_messages` — 遍历每条消息，处理 content blocks
3. `_build_base_request` — 构建 `ChatCompletionRequest`
4. `_convert_tools` — `tools` → OpenAI function calling 格式

### `_convert_message_content()`

关键设计：三种输出分流，不是全塞进 content：

```python
content_parts: list[dict[str, Any]] = []    # text + image
tool_calls: list[dict[str, Any]] = []       # tool_use → 独立字段
reasoning_parts: list[str] = []             # thinking → 独立字段

# 最终：
openai_msg["reasoning"] = reasoning          # 不在 content 里
openai_msg["tool_calls"] = tool_calls       # 不在 content 里
openai_msg["content"] = content_parts       # 只有 text + image
```

### `_convert_user_tool_result()`
tool_result → 独立 `role:"tool"` 消息（含 tool_use_id）
