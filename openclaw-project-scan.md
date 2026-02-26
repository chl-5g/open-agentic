# OpenClaw Rust 项目扫描清单（openclaw-* 模块）

> 生成时间：2026-02-26

## 1. 项目结构与主流程入口

- **Workspace members**（`Cargo.toml`）
  - `openclaw-core`
  - `openclaw-ai`
  - `openclaw-memory`
  - `openclaw-vector`
  - `openclaw-channels`
  - `openclaw-agent`
  - `openclaw-voice`
  - `openclaw-server`
  - `openclaw-cli`
  - `openclaw-canvas`
  - `openclaw-browser`
  - `openclaw-sandbox`
  - `openclaw-tools`
  - `openclaw-device`
  - `openclaw-security`
  - `openclaw-testing`

- **主要入口**
  - **CLI**：`crates/openclaw-cli/src/main.rs`，二进制名 `openclaw-rust`
    - `Gateway` 子命令会启动 `openclaw-server::gateway_service::Gateway`
    - 启动前会调用 `openclaw_device::init_device(true)` 初始化设备全局注册表
  - **HTTP Server/Gateway**：`crates/openclaw-server/src/gateway_service.rs`
    - `Gateway::new(config)`：初始化向量工厂、注册向量后端、创建 `AppContext`
    - `Gateway::start()`：
      - 初始化 DeviceManager（如果启用）
      - 启动 Orchestrator（如果启用 agents/channels/canvas）
      - 注入 AI/Security/Tools/Memory/Device ports 到 agent orchestrator
      - 按配置启用 Voice
      - 组装 Axum Router：/chat、/models、/api/channels、/api/agents、/voice/*、device 路由、agentic-rag 路由、websocket

## 2. 模块职责与依赖/被依赖关系（高层）

### 2.1 高层依赖图（概览）

- **openclaw-cli**
  - 依赖：`openclaw-server`, `openclaw-device` 以及绝大多数业务 crate
  - 作用：配置加载/生成 + 启动 gateway + 一些管理命令

- **openclaw-server**（聚合层/集成层）
  - 依赖：几乎所有模块（AI/Memory/Vector/Security/Channels/Canvas/Browser/Agent/Device/Voice/Tools/Sandbox）
  - 作用：把各模块装配进 `AppContext`，对外提供 HTTP/WebSocket/SSE API

- **openclaw-agent**（核心业务编排层）
  - 依赖：AI/Memory/Vector/Security/Voice/Channels/Tools/Sandbox/Canvas/Browser/Device
  - 作用：多智能体编排、tool 调度、安全 wrapper、session tree 等

- **openclaw-core**（基础设施/通用类型）
  - 作用：核心类型、错误、Config、Session、Message 等

- **openclaw-ai / memory / vector / channels / voice / device / security / tools / sandbox / canvas / browser**
  - 作用：各自子域能力与抽象

### 2.2 模块级清单（是否接入主流程）

- **openclaw-core**
  - **职责**：Config、错误、Message/Session 等通用抽象
  - **集成情况**：所有模块的基础依赖（已接入）

- **openclaw-ai**
  - **职责**：AI Provider 抽象 + tokenizer + tools/types
  - **集成情况**：由 `openclaw-server` 的 `ServiceFactory::create_ai_provider()` 统一创建并注入（已接入）

- **openclaw-memory**
  - **职责**：分层记忆（working/short/long）、workspace、bm25/hybrid search 等
  - **集成情况**：`openclaw-server` 创建 memory backend，并注入到 agent ports（已接入）

- **openclaw-vector**
  - **职责**：向量存储统一接口 + 多后端工厂（memory/lancedb/qdrant/pgvector/milvus/sqlite）
  - **集成情况**：
    - `Gateway::new()` 会调用 `openclaw_vector::init_all_factories()`
    - `VectorStoreRegistry` 读取 config 并创建实际 store（已接入）

- **openclaw-channels**
  - **职责**：多消息通道（telegram/discord/slack/teams/…）抽象与实现
  - **集成情况**：Orchestrator 启用 channels 时会创建 `ChannelManager` 并 `start_all()`（已接入）

- **openclaw-agent**
  - **职责**：多智能体编排与执行（orchestrator/ports/tools/safety/memory_pipeline 等）
  - **集成情况**：
    - `openclaw-server` 创建 orchestrator，并在 `Gateway::start()` 中注入 ports
    - `agents.yaml` 会在启动时 `init_agents_from_config()`（已接入）

- **openclaw-voice**
  - **职责**：STT/TTS/TalkMode/Wake
  - **集成情况**：`Gateway::start()` 根据 `config.server.enable_voice` 初始化 voice providers，并暴露 `/voice/tts`、`/voice/stt`（已接入）

- **openclaw-canvas**
  - **职责**：实时协作画布
  - **集成情况**：
    - orchestrator config 支持 enable_canvas
    - API 层按 `canvas_manager` 是否存在决定是否挂载 canvas 路由（已接入，但依赖运行期配置）

- **openclaw-browser**
  - **职责**：headless 浏览器控制（chromiumoxide）
  - **集成情况**：API 层按 `browser_config.is_some()` 决定是否挂载 browser 路由（已接入，但依赖运行期配置）

- **openclaw-tools**
  - **职责**：内置工具、技能系统、MCP、scheduler/webhook/wasm executor
  - **集成情况**：由 `openclaw-server` 创建 tool registry 注入 orchestrator/agent（已接入）

- **openclaw-sandbox**
  - **职责**：Docker/Podman/WASM sandbox + credential 安全能力
  - **集成情况**：`ServiceFactory::create_app_context()` 里按 `config.sandbox().enabled` 创建 `SandboxManager`（已接入，但依赖配置）

- **openclaw-device**
  - **职责**：设备能力节点（camera/screen/notification/system 等）+ DeviceRegistry/UnifiedDeviceManager
  - **集成情况**：
    - CLI 在启动 gateway 前强制 `init_device(true)`
    - server 侧 `DeviceManager` / `UnifiedDeviceManager` 供 API 与 Agent ports 使用（已接入）

- **openclaw-security**
  - **职责**：输入过滤、prompt 注入检测、权限控制、pipeline
  - **集成情况**：`Gateway::start()` 注入 `SecurityPort` 给 agent（已接入）

- **openclaw-testing**
  - **职责**：mock provider / mock device / mock config 等测试辅助
  - **集成情况**：当前仓库内未发现任何 `use openclaw_testing` / `openclaw_testing::` 引用（疑似“孤儿模块”或仅供外部测试使用）

## 3. 发现的问题与风险（按优先级）

### P0（高风险/影响主流程）

#### 3.1 Channels 默认注册存在竞态：注册任务被 `tokio::spawn`，未 await

- **位置**：`openclaw-server/src/orchestrator.rs`（Orchestrator 构造时）
  - 当 `enable_channels` 为 true，会 `tokio::spawn(register_default_channels(&factory))`
  - 之后 `ChannelManager::with_factory(...)` 以及 `start_all()` 可能在注册完成前执行
- **风险**：
  - 运行时出现“找不到 channel type / 没有默认通道”之类的间歇性问题
  - 环境/机器越快越不稳定（典型竞态）
- **建议修复**：
  - 将 `register_default_channels` 改为 **同步（在启动流程中 await）**：
    - 方案 A：在 `ServiceOrchestrator::start()` 中，在 `start_all()` 前 `await register_default_channels(...)`
    - 方案 B：构造 Orchestrator 时不 spawn，改为显式 init 方法并在 gateway start 中调用

#### 3.2 `openclaw-channels` 默认只注册了 telegram/discord，其它实现未接入注册表

- **位置**：`openclaw-channels/src/registry.rs`
  - `register_default_channels()` 仅调用 `register_telegram()` / `register_discord()`
- **风险**：
  - crate 内列出了大量 channel 模块（slack/teams/feishu/wecom/...），但默认流程无法创建，容易造成“实现存在但不可用”的集成缺口
- **建议修复**：
  - 明确策略：
    - 若希望默认全开：补齐所有 channel 的 register 函数并在 `register_default_channels` 中注册
    - 若希望按 feature 开关：在 `Cargo.toml` features 与注册函数之间建立对应关系（`#[cfg(feature = "slack")]` 等）

### P1（中风险/维护成本/潜在 bug）

#### 3.3 `openclaw-device` factory 注册表是全局 `Mutex<Vec<...>>`，`init_default_factory()` 可重复注册

- **位置**：`openclaw-device/src/factory.rs`
  - `static FACTORY_REGISTRY: Mutex<Vec<Arc<dyn DeviceManagerFactory>>>`
  - `init_default_factory()` 每次调用都会 push 一个 default factory
- **风险**：
  - 进程生命周期长时可能导致重复 factory，`list_factories()` 返回重复项
  - 逻辑上“注册一次”更符合预期
- **建议修复**：
  - 用 `OnceLock`/`Once` 防重复初始化
  - 或者注册前判断是否已经存在同名 factory

#### 3.4 Device 全局注册表 `init_device()` 不可重入（OnceLock set 后再调用会报错）

- **位置**：`openclaw-device/src/lib.rs`：`DEVICE_REGISTRY.set(...)`
- **现状**：CLI 启动 gateway 前调用一次即可；但如果未来 server/daemon/测试多次调用，容易踩坑
- **建议**：
  - 提供 `init_device_if_needed(print_info)` 之类的幂等初始化 API
  - 或在调用侧统一约束：只在一个入口调用

#### 3.5 `openclaw-server` 的 vector store 注册错误类型使用 `std::io::Result`，语义不匹配

- **位置**：`openclaw-server/src/vector_store_registry.rs::register_from_config` 返回 `std::io::Result<()>`
- **问题**：向量后端创建失败是“业务配置/依赖缺失”而非 IO 错误
- **建议**：
  - 返回 `openclaw_core::Result<()>`（或 `anyhow::Result<()>`），错误类型与项目保持一致
  - 现在的 `Gateway::new()` 里把错误降级为 warn 并继续，这个策略本身 OK，但错误类型建议统一

### P2（低风险/一致性/可用性）

#### 3.6 `openclaw-testing` 疑似孤儿模块

- **证据**：仓库内 grep 未发现 `openclaw_testing` 的 Rust 代码引用
- **建议**：
  - 如果只服务外部集成测试：保留，但在根 README 或 crate README 标注用途
  - 如果本仓库内部也需要：在对应 crate 的 `dev-dependencies` 引入并在 tests 使用
  - 如果已废弃：从 workspace members 移除，减少编译成本

#### 3.7 `openclaw-cli` 默认日志 filter 字符串可能不符合预期

- **位置**：`openclaw-cli/src/main.rs`：`"openclaw=debug,info"`
- **风险**：可能出现模块过滤不符合预期（取决于 tracing 的 filter 解析）
- **建议**：
  - 统一为 `"openclaw=debug,info"` 是否真的需要？更常见是 `"openclaw=debug,info"` 或 `"openclaw=debug"`
  - 建议用环境变量优先，代码里只提供最小默认值

## 4. 建议的修复路线图（可执行）

### 4.1 第一阶段（确保主流程稳定）

- **[P0]** 让 channels 注册变成可确定顺序（移除 spawn 竞态）
- **[P0]** 补齐 channels 的默认注册策略（至少：把“实现存在但未注册”的情况显式化）

### 4.2 第二阶段（幂等与初始化治理）

- **[P1]** `openclaw-device` factory 注册防重复
- **[P1]** `init_device` 幂等化或在调用侧统一约束

### 4.3 第三阶段（清理孤儿模块与工程一致性）

- **[P2]** 明确 `openclaw-testing` 的定位（保留/迁移到 tests/移出 workspace）
- **[P2]** 错误类型统一（例如 vector store registry 的返回类型）

## 5. 备注：本清单的扫描深度说明

本清单聚焦于：

- crate 级别的职责/依赖图（根据 `Cargo.toml` + `lib.rs` + server/cli 入口组装点）
- 与主流程（CLI Gateway -> Server Gateway -> Orchestrator/AppContext）相关的集成链路
- 在“集成点”上可明确识别的竞态/缺口/幂等性问题

如需进一步“深入到每个子模块业务实现细节（每个 provider/channel/tool 的具体逻辑与边界 bug）”，建议在第二轮扫描中按以下顺序：

- `openclaw-server/src/service_factory.rs`（所有组件创建与配置解析的单一入口）
- `openclaw-agent` 的 `orchestrator/router/memory_pipeline/safety`（核心业务流）
- `openclaw-tools` 的 tool registry 与 sandbox bridge（安全边界）
- `openclaw-channels` 的各 channel 实现（签名/重试/幂等/回调校验）
