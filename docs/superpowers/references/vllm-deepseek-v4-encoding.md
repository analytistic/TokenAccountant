# vLLM DeepSeek-V4 编码器

**来源:** vLLM 仓库 `vllm/tokenizers/deepseek_v4_encoding/__init__.py`
**用途:** Phase 2 审计接入调研参考 — DeepSeek-V4 的 DSML 格式编码/解码实现

## 特殊 Token

| Token | 含义 |
|---|---|
| `<｜begin▁of▁sentence｜>` | BOS |
| `<｜end▁of▁sentence｜>` | EOS |
| `<think>` / `</think>` | 推理块 |
| `<｜User｜>` | user 角色 |
| `<｜Assistant｜>` | assistant 角色 |
| `<｜DSML｜>` | DSML 命名空间 |

## 核心函数

### `encode_messages(messages, thinking_mode, ...)`
主入口。接收 OpenAI 格式消息列表，输出 DSML 格式 prompt。

流程：
1. `merge_tool_messages()` — tool 消息合并到 user 的 content_blocks
2. `sort_tool_results_by_call_order()` — 按调用顺序排序
3. 遍历消息，调用 `render_message()`

### `merge_tool_messages(messages)`
将 OpenAI 格式的 `role:"tool"` 消息合并到 user 消息的 `content_blocks` 中。

关键逻辑：
- tool 消息 → `{type:"tool_result", content:...}` 追加到同一个 user
- user 消息（紧跟在 tool 合并后的 user）→ `{type:"text", text:...}` 追加到同一个 user
- 最终同一个 user 的 content_blocks 保持：tool_result → text 顺序

### `render_message(index, messages, thinking_mode)`
渲染单条消息。

user 角色渲染：
```python
prompt += USER_SP_TOKEN
content_blocks = msg.get("content_blocks")
if content_blocks:
    parts = []
    for block in content_blocks:
        if block["type"] == "text":
            parts.append(block["text"])
        elif block["type"] == "tool_result":
            parts.append(tool_output_template.format(content=content))
    prompt += "\n\n".join(parts)
```

assistant 角色渲染：
```python
thinking_part = thinking_template.format(reasoning=reasoning) + thinking_end_token
tc_content = tool_calls_template.format(tool_calls=...)
prompt = assistant_msg_template.format(
    reasoning=thinking_part,
    content=summary_content,
    tool_calls=tc_content,
)  # 内置 eos_token
```

### `render_tools(tools)`
使用 `TOOLS_TEMPLATE` 渲染工具定义到 system prompt。

### `parse_message_from_completion_text(text, thinking_mode)`
逆解析：模型输出 → 结构化 assistant 消息（提取 reasoning、content、tool_calls）

## 关键设计决策

1. **content_blocks** — merge_tool_messages 后保持结构化数组，不展开为字符串
2. **thinking 独立** — 渲染 assistant 时由 template 控制是否输出 `<think>` 标签
3. **DSML 格式** — 工具调用使用 XML-like 标签，不是 JSON
4. **chat / thinking 双模式** — `</think>`（chat 模式，直接生成）vs `<think>`（thinking 模式，先推理）
