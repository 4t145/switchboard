# Switchboard 网关审查：`crates/controller` 与 `crates/kernel` 的关键短板

## 审查范围

本次诊断聚焦两个核心库：

- `crates/controller`
- `crates/kernel`

目标：评估 Switchboard 是否已具备“生产级万用网关”能力，并识别当前关键缺口。

## 总体结论

现有架构基础是好的：

- `controller`：控制面 API、配置解析、存储、kernel 编排
- `kernel`：数据面运行时（TCP listener、route、TLS、service 执行）以及 gRPC 控制端点

但当前阶段更接近“可运行的网关内核”，还不是“可运营的万用网关平台”。

最大的风险不在转发能力本身，而在：

1. 控制面安全
2. 配置一致性语义
3. 生命周期可靠性
4. 高可用与运维能力

## 关键问题（按优先级）

### P0 - 必须优先修复

#### 1) 控制面安全基线缺失

- Controller 管理 API 权限过高且暴露面大（`/api/kernel_manager/*`、`/api/resolve/*`、`/api/storage/*`）。
- 缺少清晰的鉴权/授权边界。
- 默认监听在较宽地址上，攻击面偏大。

参考：

- `crates/controller/src/interface/http.rs`
- `crates/controller/src/interface/http/resolve.rs`
- `crates/controller/src/interface/http/storage.rs`

#### 2) 配置下发一致性语义不足（部分成功歧义）

- Controller 在所有 kernel 确认成功前就更新本地 `current_config`。
- 一旦部分 kernel 下发失败，控制面状态可能与真实数据面状态分叉。

参考：

- `crates/controller/src/kernel/discovery.rs`
- `crates/controller/src/interface/http/kernel_manager.rs`

#### 3) Kernel listener 生命周期存在缺陷

- listener 启动后 handle 未正确持久化到 shutdown 路径。
- 优雅关闭可能无法按预期停止 controller listener。

参考：

- `crates/kernel/src/lib.rs`
- `crates/kernel/src/controller/listener.rs`

### P1 - 高优先级可靠性问题

#### 4) Kernel 地址解析与连接韧性不足

- `KernelAddr::from_str` 在 `http/https/grpc` 分支可能丢失 scheme 语义，带来 endpoint 规范化风险。
- manager 层缺少重连/退避等完整连接编排。

参考：

- `crates/controller/src/kernel.rs`
- `crates/controller/src/kernel/connection.rs`
- `crates/controller/src/kernel/grpc_client.rs`

#### 5) 发现到连接闭环不完整

- 发现机制目前以 UDS 扫描 + 手动刷新为主。
- 配置中的周期扫描等字段未充分体现在运行时行为。
- 缺少自动 connect/reconnect 与基于健康状态的生命周期管理。

参考：

- `crates/controller/src/config.rs`
- `crates/controller/src/kernel/discovery.rs`

#### 6) 单机存储限制控制面高可用

- 当前存储以本地 SurrealDB + RocksDB 为主。
- 多 controller 部署、故障切换、共享状态能力受限。

参考：

- `crates/controller/src/storage.rs`
- `crates/controller/src/storage/surrealdb_local.rs`

### P2 - 运维与平台化完整性不足

#### 7) 可观测性与审计能力不足

- 对配置发布、kernel 状态、失败归因的指标和审计事件不够完整。
- HTTP 错误映射较粗粒度。

参考：

- `crates/controller/src/interface/http.rs`
- `crates/kernel/src/controller/grpc_service.rs`

#### 8) 万用网关插件生态仍需强化契约

- resolver/provider 扩展入口已具备，但平台级防护栏仍偏薄：
  - 资源配额
  - 超时/熔断策略边界
  - 插件能力与版本治理

参考：

- `crates/controller/src/resolve.rs`
- `crates/kernel/src/registry.rs`

## 建议落地路线

### 阶段 1 - 安全与正确性基线（P0）

1. 管理面接入 authn/authz（token 或 mTLS，理想是两者都支持）。
2. 收紧默认监听策略（默认 loopback 或内网地址）。
3. 对 resolve/storage API 建立路径与 link 权限边界。
4. 修复 listener handle 生命周期问题。
5. 调整配置应用语义：
   - 要么全成功后再提交 `current_config`
   - 要么引入显式事务状态机并暴露部分失败状态。

### 阶段 2 - 可靠性与高可用（P1）

1. 完善 kernel 会话管理：
   - 重连与指数退避
   - 健康检查
   - 状态流断线重订阅
2. 完成周期发现与自动连接闭环。
3. 补齐重启场景下 stale UDS socket 处理。
4. 引入分布式/共享控制面存储选项。

### 阶段 3 - 可运营与平台化（P2）

1. 增加发布过程指标与 tracing（SLO 维度）：
   - 下发时延
   - 成功/部分成功/失败比例
   - listener 绑定失败计数
   - kernel 连接状态
2. 增加审计日志：
   - 谁改了什么
   - 何时变更
   - 目标 kernel
   - 配置版本/摘要
3. 定义插件契约与治理机制：
   - 能力模型
   - 超时/资源策略
   - 版本兼容矩阵

## 结语

Switchboard 当前已经具备良好的控制面/数据面分层和热更新运行时基础。

若要成为生产级万用网关，建议优先补齐：

1. 安全基线
2. 配置一致性语义
3. 生命周期可靠性
4. 高可用与可观测性

这些能力到位后，resolver/provider 的生态扩展才能安全放大。

## 附录：值得保留的技术亮点

- Kernel gRPC 配置更新已包含摘要校验（`bincode` + version），基础扎实。
- Controller 存储对象具备内容摘要校验，保证对象完整性。
- Kernel 事件循环中 listener 与 router 分离，有利于热更新演进。
