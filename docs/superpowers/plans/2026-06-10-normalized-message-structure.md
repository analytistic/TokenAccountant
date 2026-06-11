# NormalizedMessage 结构化改造 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 NormalizedMessage 增加 `reasoning` 和 `tool_calls` 独立字段，移除 `[thinking]` `[tool_use]` 标记转换，对齐 vLLM 的结构化数据格式。

**Architecture:** 当前将所有 block 类型（text、thinking、tool_use）展平到 `content: String`，再用标记字符串模拟结构。改为在 NormalizedMessage 中增加可选字段 `reasoning: Option<String>` 和 `tool_calls: Option<Vec<ToolCall>>`，thinking 和 tool_use 保留为独立字段，content 只包含纯文本。DeepSeek tokenizer 的 `apply_chat_template` 直接读取这些字段渲染 DSML 格式，不再需要 `preprocess_content` 标记转换。

**Tech Stack:** Rust, vLLM deepseek_v4_encoding 作为参考实现

---

### Task 1: 修改 NormalizedMessage 和 ToolCall 定义

**Files:**
- Modify: `src-tauri/src/auditor/message_converter.rs`

```rust
// 新增
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arguments: String,
}

// NormalizedMessage 修改
pub struct NormalizedMessage {
    pub role: String,
    pub content: String,               // 仅文本（不含 thinking、tool_use）
    pub reasoning: Option<String>,     // thinking block 内容
    pub tool_calls: Option<Vec<ToolCall>>,  // tool_use blocks
}
```

- [ ] **Step 1: 定义 ToolCall 结构体，修改 NormalizedMessage**

在 `message_converter.rs` 中：

- 新增 `pub struct ToolCall { pub name: String, pub arguments: String }`
- NormalizedMessage 增加 `reasoning: Option<String>` 和 `tool_calls: Option<Vec<ToolCall>>`
- `content` 语义改为：仅包含 text block 的纯文本，不包含 thinking、tool_use

- [ ] **Step 2: 更新 `content_to_text` 和 `extract_tool_result_texts`**

`content_to_text`：
- text block → 拼入 `content`（不变）
- thinking block → **不拼入 content**，留给调用方处理
- tool_use block → **不拼入 content**，留给调用方处理
- tool_result block → 跳过（由 extract_tool_result_texts 处理）

- [ ] **Step 3: 更新 `from_anthropic` 的处理逻辑**

对于 user 消息：
- `content_to_text` 提取 text → 放到 `NormalizedMessage.content`
- `extract_tool_result_texts` 提取 tool_results → 拆成 role:"tool" 消息（不变）

对于 assistant 消息：
- 遍历 blocks，按类型分：
  - `text` → 拼入 `content`
  - `thinking` → 拼入 `reasoning`
  - `tool_use` → 拼入 `tool_calls: Vec<ToolCall>`

- [ ] **Step 4: 更新 `extract_system_text`（不变）**

system 字段只含 text blocks，不受影响。

- [ ] **Step 5: 更新 debug log**

当前 log 打印 `msg.content`，改为打印 `msg.content` + `msg.reasoning` + `msg.tool_calls`。

- [ ] **Step 6: 编译验证**

Run: `cargo check`
Expected: 通过，仅 pre-existing warnings

---

### Task 2: 重构 DeepSeek tokenizer — 移除标记转换

**Files:**
- Modify: `src-tauri/src/auditor/tokenizers/deepseek.rs`

- [ ] **Step 1: 删除 `preprocess_content` 函数**

不再需要 `[thinking]` → `<think>` 和 `[tool_use: name(input)]` → DSML 的转换。

- [ ] **Step 2: 重写 `encode_messages` 中 assistant 分支**

当前：
```rust
"assistant" => {
    prompt.push_str(&processed);
    prompt.push_str("<｜end▁of▁sentence｜>");
}
```

改为直接读取结构化字段：
```rust
"assistant" => {
    // thinking part
    if let Some(ref r) = msg.reasoning {
        prompt.push_str("<think>");
        prompt.push_str(r);
        prompt.push_str("</think>");
    }
    // content
    prompt.push_str(&msg.content);
    // tool calls (DSML)
    if let Some(ref tcs) = msg.tool_calls {
        for tc in tcs {
            prompt.push_str(&format!(
                "\n\n<｜DSML｜tool_calls>\n<｜DSML｜invoke name=\"{}\">\n<｜DSML｜parameter name=\"arguments\" string=\"true\">{}</｜DSML｜parameter>\n</｜DSML｜invoke>\n</｜DSML｜tool_calls>",
                tc.name, tc.arguments
            ));
        }
    }
    prompt.push_str("<｜end▁of▁sentence｜>");
}
```

- [ ] **Step 3: 更新 user 分支 — 移除 `processed`（不再需要 preprocess）**

`processed` 变量改为直接使用 `msg.content`，因为 content 现在只有纯文本。

- [ ] **Step 4: 编译验证**

Run: `cargo check`
Expected: 通过

---

### Task 3: 清理 GptTokenizer 和 ClaudeTokenizer

**Files:**
- Modify: `src-tauri/src/auditor/tokenizers/gpt.rs`
- Modify: `src-tauri/src/auditor/tokenizers/claude.rs`

- [ ] **Step 1: GptTokenizer 加 reasoning 和 tool_calls 处理**

当前 `apply_chat_template` 只拼接 `msg.content`。改为把 `reasoning` 和 `tool_calls` 也拼进去（保持兜底兼容）：

```rust
fn apply_chat_template(&self, conv: &Conversation) -> String {
    let mut text = String::new();
    for msg in &conv.messages {
        if let Some(ref r) = msg.reasoning {
            text.push_str(r);
            text.push('\n');
        }
        text.push_str(&msg.content);
        text.push('\n');
        if let Some(ref tcs) = msg.tool_calls {
            for tc in tcs {
                text.push_str(&format!("[tool_use: {}({})]\n", tc.name, tc.arguments));
            }
        }
    }
    for tool in &conv.tools {
        text.push_str(&format!("tool: {}\ndescription: {}\nschema: {}\n\n",
            tool.name, tool.description, tool.input_schema));
    }
    text
}
```

- [ ] **Step 2: 编译验证**

Run: `cargo check`
Expected: 通过

---

### Task 4: 更新调试 log 和确认输出

**Files:**
- Modify: `src-tauri/src/auditor/message_converter.rs`

- [ ] **Step 1: 更新 `from_anthropic` 的 tracing log**

输出每条 msg 的 role、content 长度、reasoning 是否存在、tool_calls 数量。

- [ ] **Step 2: 编译运行验证**

Run: `cargo run`
Expected: 应用正常启动。发送请求后，日志显示新的结构化字段。

---

## Self-Review

### Spec Coverage

| Requirement | Task |
|---|---|
| NormalizedMessage 加 reasoning 字段 | Task 1 |
| NormalizedMessage 加 tool_calls 字段 | Task 1 |
| 移除 `[thinking]` 标记 | Task 2 |
| 移除 `[tool_use]` 标记 | Task 2 |
| DeepSeek DSML 渲染从结构化字段读取 | Task 2 |
| GptTokenizer 兜底兼容 | Task 3 |
| Message_converter 输出结构化数据 | Task 1 |
| 清理 preprocess_content | Task 2 |

### Placeholder Check
- 无 "TBD", "TODO", "implement later"
- 每步有完整代码或明确描述

### Type Consistency
- `ToolCall` 的 `name: String`, `arguments: String` 与 DeepSeek DSML 格式一致
- `NormalizedMessage.reasoning: Option<String>` 与 vLLM 的 `reasoning` 字段对应
- `NormalizedMessage.tool_calls: Option<Vec<ToolCall>>` 与 vLLM 的 `tool_calls` 字段对应
- `content` 只含 text block 文本，语义与 OpenAI `content` 一致

---

## Post-Implementation Findings

### Issue 1: DSML `<｜Assistant｜>` transition 位置和 thinking 标签逻辑错误

**发现时间:** 2026-06-10 Debug/验收阶段

**问题:** 最初 `encode_messages` 在 user 分支写死了 `<｜Assistant｜></think>`，assistant 分支又单独加 `<think>`/`</think>`，产生 `</think><think>` 无效序列。

**修复:** 按 vLLM 架构重构 `encode_messages`：
- user 消息: `<｜User｜>{content}` + transition `<｜Assistant｜>` + `<think>`/`</think>`
  - transition 取决于**下一条消息**（assistant）是否有 `reasoning`
  - 有 reasoning：`<think>` → 后续 assistant 输出 `{reasoning}</think>`
  - 无 reasoning：`</think>` → 后续 assistant 直接输出 `{content}`
- assistant 消息: `{reasoning}</think>{content}{tool_calls}<｜end▁of▁sentence｜>`
  - `<think>` 来自 user→assistant transition，assistant 自身只加 `</think>`
  - 严格匹配 vLLM 的 `thinking_template + assistant_msg_template`

**验证依据:** 对照 vLLM 源码 `deepseek_v4_encoding/__init__.py` 的 `render_message` 和 `encode_messages`。
