# TokenAccountant 架构设计方案

## 项目目标

构建一个开源工具，帮助用户检测 API 中转站是否虚报 token 用量。支持token prefill（input）和decode（output）中的命中缓存和未命中缓存计算、并提供Web 可视化监控界面。实现上参考cc swicth 提供命令行命令启动，与前端启动。

---

## 核心需求（已确认）

1. **多模型支持**：自动识别 qwen、glm、minimax、gemini、gpt、mimo、deepseek 等模型及 API 格式
2. **产品形态**：发布到 GitHub，用户安装后通过cli命令启动（后续可以发展成app），并可以启动 Web 前端进行可视化监控和配置
3. **检测粒度**：区分 prefill 阶段，以及 decode 阶段的 token 统计（缓存命中/未命中）
4. **缓存检测**：通过 Token IDs 最长前缀匹配，计算token详细数量（这一块可能需要调研vllm的缓存方式 做的更加精确）
5. **持久化**：支持 Resume 场景（跨会话检测缓存命中）需要将prefix结构存储，而不是只放在内存中

---

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                    TokenAccountant                          │
│  ┌─────────────────┐  ┌──────────────────┐  ┌───────────┐ │
│  │  Web 前端       │  │  后端 API 服务   │  │  claude    │ │
│  │  (Tauri/React) │◄─┤  (Axum + Rust) │◄─┤               │
│  └────────┬────────┘  └────────┬─────────┘  └───────────┘ │
│           │                     │                           │
│           │          ┌──────────┴──────────┐               │
│           │          │  代理拦截层         │               │
│           │          │  (Axum HTTP Proxy) │               │
│           │          └──────────┬──────────┘               │
│           │                     │                           │
│  ┌────────┴────────────────────┴──────────────────┐      │
│  │            Token 计算引擎                        │      │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐ │      │
│  │  │ Tiktoken │  │ TF/Python│  │ Cache    │ │      │
│  │  │ -rs      │  │ Script   │  │ Detector │ │      │
│  │  │(OpenAI/  │  │(Other    │  │(vllm类似匹配机制     │ │      │
│  │  │ Qwen/    │  │ Models)  │  │  +  │ │      │
│  │  │ DeepSeek)│  │          │  │ LRU)     │ │      │
│  │  └──────────┘  └──────────┘  └──────────┘ │      │
│  └─────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
                    ┌───────────────┐
                    │  中转站 API   │
                    └───────────────┘
```

### 核心模块

#### 1. 启动时检查并修改 settings.json（需要进一步调研）

**功能**：自动修改 `~/.claude/settings.json`，将 `ANTHROPIC_BASE_URL` 指向本地代理。

**实现**（Rust）：

**注意**：Claude Code 启动时只读一次配置，所以是否要提示重启claude需要考虑

这只是一个初步想法，具体 我们得参考cc switch是怎么执行流程 的 因为还涉及用户的中转站api model key等信息录入

---

#### 2. HTTP 代理拦截层（Axum）转发api 与 计算tokens（prefill部分）大致思路确认 需要思考具体细节

**功能**：拦截所有通过 TokenAccountant 的 API 请求和响应，转发到真实中转站。

**实现**（Axum + tokio）：

**关键特性**：

- ✅ **异步并行**：分词计算和 API 调用并行，不增加延迟
- ✅ **支持流式响应**（SSE/Streaming）
- ✅ **自动识别模型**（根据 URL 路径和请求体中的 `model` 字段）

但是这里的具体线程协作，哪里需要上锁 哪里会导致出错 需要仔细思考

---

#### 3. 模型识别器

**功能**：根据 HTTP 请求特征自动判断模型类型和 API 格式。

**识别规则**：

| 特征                                                                                                                             | 判断结果   |
| -------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| 对于api格式其实不好判断，有些中转会有 `https://api.deepseek.com/anthropic`（deepseek的官方格式）所以从url判断api格式不好操作  |            |
| 需不需要判断api格式是个问题 其实无非就是anthropic和openai两种                                                                    |            |
| `model` 字段包含 可以判断是什么模型                                                                                            | OpenAI GPT |
|                                                                                                                                  | Claude     |
|                                                                                                                                  | Gemini     |
|                                                                                                                                  | Qwen       |
|                                                                                                                                  | GLM        |
|                                                                                                                                  | MiniMax    |
|                                                                                                                                  | MiMo       |
|                                                                                                                                  | DeepSeek   |

---

#### 4. Token 计算引擎（模块化设计）

**功能**：根据模型类型，使用对应的分词器本地计算 token 数。

**模块化设计**：需要注意模块化，特别有些分词器可能只有python实现 这里需要注意多语言如何一起构造项目

**分词器选择策略**：

需要调研哪些是公开分词器，哪些有rust实现 哪些只有python实现 哪一些不公开只能通过sdk（官方）实现

#### 5. 缓存检测器（前缀匹配 + LRU 驱逐）

**功能**：通过 Token IDs 最长前缀匹配，检测中转站是否虚报缓存命中。

**核心算法**（模仿 vLLM 的 Prefix Caching）：

##### 5.1 数据结构调整（模仿 vLLM）

##### 5.2 缓存检测流程

得到prefill（input）的命中和未命中token数统计

#### 6. 返回信息检测（需要调研 或者实验 decode的信息是如何返回的，流式还是rollout完返回）

当得到返回信息时，可以拿到decode（output）的message，这样通过分词器可以计算出decode （output）阶段的token数

然后返回的信息中一般会有token 计数信息，但是我们要注意 我不太清楚claude在请求的时候 返回信息是不是流式的 还是rollout结束后 才会被 推理段->中转站->本地截获->claude 这个主要是影响我们统计中 匹配prefill的是哪一个。然后我们就可以计算出本地分词器计算的prefill 命中 未命中 decode的tokens 和中转站返回的。

#### 7. 信息对比

时刻计算本地的 和中转的 得到web面板监控数据。

#### 8. Web 前端（Tauri + React）

**功能**：提供可视化监控和配置界面。

**技术栈**：

- **前端**：React + TypeScript + Tailwind CSS
- **后端**：Tauri（Rust）+ Axum（Web API）
- **图表**：Chart.js 或 ECharts

**核心页面**：

1. **仪表盘**：

   分prefill 和 decode两个面板

   各自显示本地计算 和 中转站返回的信息统计

   可以是曲线，显示一个时间窗口内的统计，搭配柱状图显示这个中转站的（每个柱子是一天内的统计）
2. **配置页面**：（可以先不做）

   - 中转站 URL 配置
   - 模型配置（API Key、分词器选择）
   - 告警阈值配置（默认 5%）

---

## 技术栈选型（还没确认，可以讨论给出建议）

| 层级                 | 技术选型                                                      | 理由                              |
| -------------------- | ------------------------------------------------------------- | --------------------------------- |
| **后端框架**   | Axum（Rust）                                                  | 高性能，异步支持好，适合代理场景  |
| **HTTP 代理**  | Axum + tokio                                                  | 异步，支持流式响应                |
| **前端框架**   | Tauri + React + TypeScript                                    | 跨平台桌面应用，和 CC Switch 一致 |
| **数据库**     | SQLite (`rusqlite`)                                         | 轻量级，无需额外安装              |
| **Token 计算** | `tiktoken-rs`（GPT/Qwen/DeepSeek）、Python 脚本（其他模型） | 精确 + 近似                       |
| **缓存检测**   | 自定义 Trie 树 + LRU 驱逐（模仿 vLLM）                        | 精确检测缓存命中                  |
| **WebSocket**  | `tokio-websockets`                                          | 实时推送监控数据                  |

---

## 实现优先级

### Phase 1: MVP（最小可行产品）

- [ ] 启动时检查并修改 `settings.json`
- [ ] HTTP 代理拦截层（基础版，支持 OpenAI 格式）
- [ ] 模型识别器（支持 OpenAI GPT、Qwen、DeepSeek）
- [ ] Token 计算引擎（支持 `tiktoken-rs`）
- [ ] 差异对比器（基础对比）
- [ ] SQLite 数据存储
- [ ] 简单的 Web 前端（请求列表 + 差异显示）

### Phase 2: 完整功能

- [ ] 支持 Claude、Gemini 格式
- [ ] 支持 GLM、MiniMax、MiMo
- [ ] 缓存检测（Trie 树 + LRU 驱逐）
- [ ] 可疑请求标记 + 告警
- [ ] 数据统计聚合
- [ ] 持久化（服务关闭时写入硬盘，启动时加载）

### Phase 3: 高级功能

- [ ] 实时 WebSocket 推送
- [ ] 导出报告（CSV/PDF）
- [ ] 多用户支持
- [ ] Docker 部署

---

## 关键挑战

### 1. 分词器精度

**问题**：不同分词器rust实现和python实现 部分是没有公开的，需要调用官方sdk或者根本不公开，我们先做公开能用的，然后把其他的先用待实现接口形式

### 2. HTTPS 拦截

**问题**：中转站 API 通常走 HTTPS，需要解密才能看到请求/响应内容。

**解决方案**：

- **不需要解密**！用户直接在代码里设置 `base_url="http://localhost:8080/v1"` 即可
- TokenAccountant 作为**反向代理**，接收 HTTP 请求，转发到 HTTPS 的中转站

### 3. 流式响应处理

**问题**：API 响应可能是流式（SSE/Streaming），如何拦截并解析？

**解决方案**：

- Axum 支持流式响应（`StreamBody`）
- 在流式传输过程中，实时计算 token 数（可能需要将流式decode的和对应prefill对应上，或者是不是其实不需要对应？）

---

## 项目结构（可以修改提意见 注意模块化 多分文件夹 不要一个文件上千行）

```
tokenaccountant/
├── Cargo.toml
├── src/
│   ├── main.rs              # 入口
│   ├── proxy.rs             # HTTP 代理（Axum）
│   ├── tokenizer/           # 分词器模块（模块化）
│   │   ├── mod.rs          # 分词器 trait + 工厂
│   │   ├── gpt.rs         # GPT 分词器（tiktoken-rs）
│   │   ├── qwen.rs        # Qwen 分词器（tiktoken-rs）
│   │   ├── deepseek.rs    # DeepSeek 分词器（tiktoken-rs）
│   │   ├── claude.rs      # Claude 分词器（近似）
│   │   └── python.rs      # 其他模型分词器（调用 Python 脚本）
│   ├── cache_detector/   # 缓存检测器（前缀匹配+ LRU 驱逐）
|.  |--....其他 
│   ├── config.rs           # 配置管理（读写 config.toml）
│   └── api/              # Web API（供前端调用）
├── frontend/               # Tauri 前端（React + TypeScript）
│   ├── src/
│   │   ├── components/
│   │   │   ├── Dashboard.tsx
│   │   │   ├── RequestList.tsx
│   │   │   └── StatsChart.tsx
│   │   └── App.tsx
│   └── package.json
├── python/
│   ├── tokenize.py         # Python 分词器脚本
│   └── requirements.txt
├── config.toml            # 配置文件
└── tests/
```

---
