# PRD: TokenAccountant v0.2 审计接入

**状态**: ✅ 已完成  
**版本**: v0.2  
**发布日期**: 2026-06-11  
**作者**: Product Team  
**最后更新**: 2026-06-11

---

## 📋 目录

1. [背景与问题](#背景与问题)
2. [目标](#目标)
3. [非目标 (Non-goals)](#非目标-non-goals)
4. [用户故事](#用户故事)
5. [功能需求](#功能需求)
6. [非功能需求](#非功能需求)
7. [成功指标](#成功指标)
8. [风险与缓解](#风险与缓解)
9. [发布标准](#发布标准)

---

## 背景与问题

### v0.1 的局限性

v0.1 实现了基础的代理和 Token 计数功能，但存在以下问题：

1. **审计能力弱**：只能记录基础的 Input/Output Token，无法检测 Provider 是否"虚报" Token 数
2. **流式响应体验差**：早期版本需要等全部响应收完再返回，导致客户端等待时间长
3. **Cache 检测不准确**：无法有效检测 Prefix Cache 是否命中，影响成本优化判断
4. **缺乏开发调试工具**：开发者无法直观看到 render 前后的文本差异
5. **消息格式不统一**：不同 Provider 的消息格式差异导致 Token 计数偏差

### 核心问题

**如何验证 Provider 返回的 Token 数是否准确？**

这是用户最核心的痛点。如果 Provider 虚报 Token 数，用户会被多收费，但难以发现。

### 为什么现在做

- v0.1 已完成基础架构，可以基于此构建增强审计能力
- 用户在 v0.1 使用中反馈"无法判断 Provider 是否可信"
- Prefix Cache 是降低成本的利器，但需要工具来验证其是否生效

---

## 目标

### 产品目标

1. **增强审计能力**：不仅记录 Token 数，还要检测其真实性（通过 Input/Output 文本对比）
2. **实时流式转发**：SSE 逐 chunk 转发，提升用户体验
3. **Cache 命中检测**：通过 block hash chain 检测 Prefix Cache 是否命中
4. **开发者友好**：提供 Render Inspector 面板，帮助开发者调试
5. **格式统一**：消息转换与 vLLM 对齐，确保 Token 计数准确

### 业务目标

- 提升用户信任度（通过准确的审计能力）
- 收集更多用户反馈，为 v0.3 功能规划提供依据
- 建立技术壁垒（审计算法 + Cache 检测）

---

## 非目标 (Non-goals)

**v0.2 明确不做的事情**：

1. ❌ **不做自动成本优化**：不自动切换 Provider 或调整 Prompt
2. ❌ **不做高级数据分析**：不提供趋势图、异常检测等（留给 v0.3）
3. ❌ **不做多用户支持**：仍然是单用户应用
4. ❌ **不做云端功能**：所有功能本地运行
5. ❌ **不做非技术用户的简化模式**：目标用户仍是开发者

---

## 用户故事

### 核心用户故事

**作为开发者**，我希望能够：

1. **检测 Token 计数真实性**
   - 作为开发者，我希望能够看到本地计算的 Token 数与 Provider 声称的 Token 数的差异，以便判断 Provider 是否可信
   - 作为开发者，我希望系统能够自动标记"可疑"请求（差异超过阈值），以便快速定位问题

2. **实时监控流式响应**
   - 作为开发者，我希望代理能够实时转发 SSE 流式响应（逐 chunk），以便使用体验与直连一致
   - 作为开发者，我希望在流式转发过程中仍能进行审计，以便不牺牲准确性

3. **验证 Prefix Cache 是否命中**
   - 作为开发者，我希望能够知道某次请求是否命中了 Prefix Cache，以便优化我的 Prompt 结构
   - 作为开发者，我希望看到 Cache 命中的 block 信息，以便理解 Cache 策略

4. **调试消息格式和 Token 计数**
   - 作为开发者，我希望能够看到 Render 前后的文本对比（开发者模式），以便验证 Tokenizer 是否正确处理了我的 Prompt
   - 作为开发者，我希望能够查看每次请求的详细审计记录（包括差异原因），以便排查问题

5. **支持更多模型和格式**
   - 作为开发者，我希望 TokenAccountant 能够正确处理 DeepSeek-V4 的 DSML 格式，以便使用最新的模型
   - 作为开发者，我希望消息转换逻辑与 vLLM 对齐，以便 Token 计数与我的推理后端一致

---

## 功能需求

### 1. StreamForwarder — SSE 逐 chunk 转发

**描述**：重构代理转发逻辑，支持 SSE (Server-sent Events) 逐 chunk 转发，同时进行审计。

**功能点**：

| 功能 | 说明 | 优先级 |
|------|------|--------|
| SSE 流式解析 | 实时解析 Provider 返回的 SSE 流 | P0 |
| 逐 chunk 转发 | 收到一个 chunk 就立即转发给客户端 | P0 |
| 审计管道接入 | 在转发过程中收集数据，最终存入审计记录 | P0 |
| 错误处理 | SSE 流异常时能够正确关闭连接 | P1 |
| 超时处理 | 长时间无数据时能够超时断开 | P2 |

**技术方案**：
- 使用 `reqwest` 的 `stream` 功能读取响应体
- 使用 `futures-util` 的 `StreamExt` 逐 chunk 处理
- 审计数据先缓存在内存，请求结束后写入数据库

**验收标准**：
- 流式响应延迟 < 100ms（从收到 Provider chunk 到转发给客户端）
- 审计数据不丢失（即使客户端提前断开）
- 支持 Claude、OpenAI、DeepSeek 的 SSE 格式

### 2. 审计管道 (Audit Pipeline)

**描述**：在请求/响应生命周期中插入审计逻辑，检测 Token 数真实性。

**功能点**：

| 功能 | 说明 | 优先级 |
|------|------|--------|
| Input 文本捕获 | 从请求中提取用于 Token 计数的文本 | P0 |
| Output 文本捕获 | 从响应中提取 Assistant 的回复文本 | P0 |
| Token 数对比 | 本地计算 vs Provider 声称 | P0 |
| 差异计算 | 计算绝对值和百分比差异 | P0 |
| 可疑标记 | 差异超过阈值自动标记为"suspicious" | P0 |
| 差异原因推断 | 自动推断差异原因（如：图片 block 未计入） | P1 |

**审计记录字段**：

```sql
CREATE TABLE audit_logs (
    id TEXT PRIMARY KEY,
    timestamp INTEGER,
    provider_id TEXT,
    model TEXT,
    input_tokens_claimed INTEGER,
    output_tokens_claimed INTEGER,
    input_tokens_calculated INTEGER,
    output_tokens_calculated INTEGER,
    input_tokens_diff INTEGER,
    output_tokens_diff INTEGER,
    is_suspicious BOOLEAN,
    suspicious_reason TEXT,
    request_id TEXT,
    session_id TEXT
);
```

**验收标准**：
- 审计记录完整（所有字段正确填充）
- 可疑检测准确率 > 90%（人工验证 100 条记录）
- 性能影响 < 5%（对比关闭审计的情况）

### 3. CacheDetector — Prefix Cache 命中检测

**描述**：通过 block hash chain 检测 Prefix Cache 是否命中。

**功能点**：

| 功能 | 说明 | 优先级 |
|------|------|--------|
| Block 提取 | 从请求中提取 message blocks | P0 |
| Hash 计算 | 为每个 block 计算 hash | P0 |
| Store | 将 hash 存储到内存 cache | P0 |
| Detect | 新请求到来时检测是否有 block hash 匹配 | P0 |
| 命中率统计 | 记录 cache 命中次数和未命中次数 | P1 |
| 跨会话持久化 | 将 cache 数据持久化到磁盘（v0.3 功能，v0.2 仅内存） | P2 |

**技术方案**：
- 使用 `sha2` crate 计算 block 文本的 SHA-256 hash
- 使用 LRU Cache 存储最近的 block hashes
- 检测逻辑：如果新请求的某个 block hash 在 cache 中，则认为可能命中了 Prefix Cache

**验收标准**：
- Cache 检测准确率 > 80%（与手动验证对比）
- 误报率 < 10%（将未命中判断为命中）
- 性能影响 < 3%（hash 计算 + cache 查找）

### 4. DiffComparator — 差异对比器

**功能点**：

| 功能 | 说明 | 优先级 |
|------|------|--------|
| Input 文本对比 | 对比"用于 Token 计数的文本"与"Provider 收到的文本" | P0 |
| Output 文本对比 | 对比"本地计算的 Output"与"Provider 返回的 Output" | P0 |
| 差异高亮 | 在 UI 中高亮显示差异部分 | P1 |
| 差异原因标注 | 自动标注常见差异原因（如：system message 被忽略） | P1 |

### 5. Render Inspector — 开发者面板

**描述**：开发者模式下，展示 render 前后的文本对比，帮助验证 Tokenizer 是否正确处理了 Prompt。

**功能点**：

| 功能 | 说明 | 优先级 |
|------|------|--------|
| 开发者模式开关 | 在设置中开启/关闭开发者模式 | P0 |
| Render 前文本 | 展示 Tokenizer 收到的原始文本 | P0 |
| Render 后文本 | 展示 Tokenizer 处理后的 tokens | P0 |
| 文本对比视图 | 并排或上下对比，高亮差异 | P0 |
| TemplateParams | 支持参数化模板（如：`<user>{{name}}</user>`） | P1 |
| 复制到剪贴板 | 方便开发者复制文本进行调试 | P2 |

**UI 设计**：
- 在请求详情弹窗中增加"Render Inspector"标签页
- 使用 Monaco Editor 或 CodeMirror 展示文本（支持语法高亮）
- 差异部分用红色/绿色高亮

**验收标准**：
- 开发者模式默认关闭，开启后不影响普通用户
- Render Inspector 数据实时更新（每次请求后刷新）
- 支持大文本（> 100KB）的展示（虚拟滚动）

### 6. Message Converter — 消息格式转换器

**描述**：将不同 Provider 的消息格式统一转换为内部格式，确保 Token 计数准确。

**功能点**：

| 功能 | 说明 | 优先级 |
|------|------|--------|
| Claude 格式解析 | 解析 Claude 的 `messages` 格式 | P0 |
| OpenAI 格式解析 | 解析 OpenAI 的 `messages` 格式 | P0 |
| DeepSeek 格式解析 | 解析 DeepSeek 的 `messages` + DSML 格式 | P0 |
| vLLM 格式对齐 | 转换逻辑与 vLLM 的 `tokenizer_manager.py` 对齐 | P0 |
| Tool Calling 支持 | 正确处理 tool calls 的 token 计数 | P0 |
| Thinking Mode 支持 | 正确处理 DeepSeek 的 thinking blocks | P0 |

**vLLM 对齐关键点**：
- System message 的处理方式
- Tool schema 的 token 计数方式
- Multi-modal (图片) 的 token 计数方式

**验收标准**：
- 与 vLLM 的 Token 计数差异 < 5%（测试 100 条真实请求）
- 支持 DeepSeek-V4 的所有 DSML 特性

### 7. 请求列表页增强

**功能点**：

| 功能 | 说明 | 优先级 |
|------|------|--------|
| 可疑请求高亮 | 可疑请求用红色背景，正常请求灰色 | P0 |
| 筛选器 | 按 Provider、模型、可疑状态筛选 | P0 |
| 搜索 | 按请求 ID 或 session ID 搜索 | P1 |
| 分页 | 支持分页加载（防止数据量过大） | P1 |
| 排序 | 按时间、Token 数、差异值排序 | P1 |
| 详情弹窗 | 点击某行展开详情（包括 Render Inspector） | P0 |

---

## 非功能需求

### 性能

- SSE 逐 chunk 转发的延迟增加 < 50ms
- 审计管道的性能影响 < 5%
- Cache 检测的性能影响 < 3%
- 请求列表页加载 1000 条记录 < 1 秒

### 可靠性

- 审计数据不丢失（即使客户端提前断开）
- Cache 数据内存占用 < 50MB（1000 个 blocks）
- Render Inspector 不会导致应用卡顿

### 安全性

- 开发者模式需要明确开启，不会泄露敏感信息给普通用户
- 审计记录中的 API Key 等敏感信息需要脱敏

### 兼容性

- 支持 Claude、OpenAI、DeepSeek 的 SSE 格式
- 支持 DeepSeek-V4 的 DSML 格式
- 消息转换逻辑与 vLLM 0.6+ 对齐

---

## 成功指标

### 核心指标

| 指标 | 目标值 | 测量方式 |
|------|--------|----------|
| 审计记录完整率 | > 99% | 数据库记录数 / 代理请求数 |
| 可疑检测准确率 | > 90% | 人工验证 100 条记录 |
| SSE 转发延迟 | < 100ms | 自动化测试 |
| 用户满意度 (NPS) | > 50 | 用户反馈问卷 |

### 次要指标

| 指标 | 目标值 | 测量方式 |
|------|--------|----------|
| Cache 检测准确率 | > 80% | 人工验证 |
| vLLM 对齐准确率 | > 95% | 对比 vLLM 的 Token 计数 |
| Bug 报告数 | < 3/周 | GitHub Issues |

---

## 风险与缓解

### 技术风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| SSE 流式转发复杂 | 高 | 参考开源实现（如 `tower-http` 的 SSE 支持） |
| vLLM 对齐困难 | 中 | 先支持主流格式，逐步完善边缘情况 |
| 性能退化 | 中 | 性能测试覆盖所有新增功能 |
| DeepSeek-V4 DSML 格式变化 | 中 | 与 DeepSeek 团队保持沟通 |

### 产品风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 审计结果误报 | 高 | 提供详细差异原因，让用户自己判断 |
| 开发者模式复杂 | 中 | 默认隐藏，仅在开发者模式下展示 |
| 用户不理解 Prefix Cache | 低 | 提供文档说明 |

---

## 发布标准

### v0.2 发布检查清单

**功能完整性**：
- [x] StreamForwarder SSE 逐 chunk 转发
- [x] 审计管道 (input/output token 检测 + diff)
- [x] 审计记录 SQLite 存储
- [x] CacheDetector block hash chain
- [x] message_converter 与 vLLM 格式对齐
- [x] DeepSeek-V4 DSML 模板匹配官方
- [x] Render Inspector 开发者面板
- [x] TemplateParams 参数化

**质量保障**：
- [x] 手动测试通过（核心流程）
- [x] 性能测试通过（SSE 延迟、审计性能影响）
- [x] 已知 Bug 已修复或记录

**文档完整性**：
- [x] Changelog 已更新
- [x] 用户手册（新增功能说明）

**已知限制（留到 v0.3 解决）**：
- ⚠️ **Token 审计精确性** - 当前为"大致精确"，后续需要更多逻辑保证精确（如：边缘情况处理、更多模型适配、精度验证）

---

## 附录

### 审计管道数据流

```
Client Request
    ↓
Proxy Server (Axum)
    ↓
Message Converter (统一格式)
    ↓
Tokenizer (计算 Input Token)
    ↓
Forward to Provider API
    ↓
Stream Forwarder (逐 chunk 转发)
    ↓
Tokenizer (计算 Output Token)
    ↓
Diff Comparator (对比差异)
    ↓
Cache Detector (检测 Cache 命中)
    ↓
Audit Logger (写入 SQLite)
```

### Render Inspector 工作流程

1. 用户开启"开发者模式"
2. 每次请求完成后，Tokenizer 将"渲染前文本"和"tokens"存入 DevTraceBuffer
3. 用户点击某条请求，查看详情
4. 在"Render Inspector"标签页中，并排展示"渲染前"和"tokens"
5. 差异部分高亮显示

---

**变更记录**：

| 日期 | 变更内容 | 作者 |
|------|----------|------|
| 2026-06-11 | 初始化 PRD | Product Team |
| 2026-06-11 | 补充完整内容 | AI Assistant |

---

**批准签字**：

- 产品负责人：\_\_\_\_\_\_\_\_\_\_
- 技术负责人：\_\_\_\_\_\_\_\_\_\_
- 发布日期：2026-06-11
