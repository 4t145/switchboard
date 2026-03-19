# Gateway API 实现者指南（中文翻译）

> Source: `https://gateway-api.sigs.k8s.io/guides/implementers/`
>
> Note: This is an unofficial Chinese translation for engineering reference.

## 目录

- [实现 Gateway API 时需要牢记的重要事项](#实现-gateway-api-时需要牢记的重要事项)
  - [Gateway API 是 `kubernetes.io` API](#gateway-api-是-kubernetesi-o-api)
  - [Gateway API 通过 CRD 交付](#gateway-api-通过-crd-交付)
  - [标准通道 CRD 的变更具备向后兼容性](#标准通道-crd-的变更具备向后兼容性)
- [实现规则与指南](#实现规则与指南)
  - [CRD 管理](#crd-管理)
  - [一致性（Conformance）与版本兼容性](#一致性conformance与版本兼容性)
    - [版本兼容性](#版本兼容性)
  - [标准状态字段与 Conditions](#标准状态字段与-conditions)
- [TLS](#tls)
  - [Listener 隔离](#listener-隔离)
  - [间接配置](#间接配置)
    - [1. 来自外部系统的证书](#1-来自外部系统的证书)
    - [2. 自动生成并后续填充的 TLS 证书](#2-自动生成并后续填充的-tls-证书)
    - [3. 由其他角色指定的证书](#3-由其他角色指定的证书)
  - [TLS 扩展的总体指导原则](#tls-扩展的总体指导原则)
- [资源细节](#资源细节)
  - [GatewayClass](#gatewayclass)
  - [Gateway](#gateway)
  - [Routes](#routes)
    - [HTTPRoute](#httproute)
    - [TLSRoute](#tlsroute)
    - [TCPRoute](#tcproute)
    - [UDPRoute](#udproute)
  - [ReferenceGrant](#referencegrant)

## 实现者指南

这是一个收集经验的文档，面向“**实现 Gateway API**”的工程师，整理那些不适合直接写进类型 godoc 的技巧、注意事项与常见坑。

如果你是 API 的最终使用者，而不是实现者，这份文档的相关性可能有限。

这是一份持续演进的文档，如有缺失，欢迎通过 PR 补充。

## 实现 Gateway API 时需要牢记的重要事项

这些点大多不令人意外，但它们在实现中常常有不那么直观的影响。

### Gateway API 是 `kubernetes.io` API

Gateway API 使用 `gateway.networking.k8s.io` API Group。这意味着和 Kubernetes 核心二进制中交付的 API 一样，每次发布都会经过上游 Kubernetes 审核者评审。

### Gateway API 通过 CRD 交付

Gateway API 以一组 CRD 形式提供，版本管理遵循其 [Versioning Policy](https://gateway-api.sigs.k8s.io/concepts/versioning/)。

最关键的一点是：看起来“相同”的对象（相同的 `group`、`version`、`kind`）也可能存在轻微的 Schema 差异。Gateway API 会以兼容方式演进，通常可以“直接工作”，但实现侧仍应采取一些措施来提升可靠性。

另外，由于通过 CRD 交付，如果在集群尚未安装相关 CRD 时就尝试 `get/list/watch` Gateway API 资源，Kubernetes 客户端很可能会返回严重错误。

Gateway API 的 CRD 都包含两个特定注解：

- `gateway.networking.k8s.io/bundle-version: <semver-release-version>`
- `gateway.networking.k8s.io/channel: <channel-name>`

其中 bundle version 与 channel（发布通道）的概念见版本文档。实现可以利用这些注解判断当前集群安装了哪些 Schema 版本（如果有）。

### 标准通道 CRD 的变更具备向后兼容性

标准通道（Standard Channel）CRD 的契约之一是：**同一 API 版本内的变更必须兼容**。注意，实验通道（Experimental Channel）不提供向后兼容保证。

Gateway API 的版本策略总体对齐 Kubernetes 上游，但允许“校正验证规则”。例如：规范明确某值非法，但旧验证未覆盖该场景，未来版本可能补上校验并阻止该非法输入。

该兼容契约还意味着：

- 实现使用较低版本 Schema 编写时，面对更高版本 CRD 一般不会直接失效，因为新 Schema 应可序列化为实现代码使用的旧版本结构。
- 实现使用较高版本编写时，那些“仅高版本支持”的新字段在旧集群中不会出现，因此只是“不会被用到”。

## 实现规则与指南

### CRD 管理

关于如何管理 Gateway API CRD（包括何时可以把 CRD 安装与实现一起打包），请参考 [CRD Management Guide](https://gateway-api.sigs.k8s.io/guides/crd-management/)。

### 一致性（Conformance）与版本兼容性

一个“符合规范”的 Gateway API 实现，是指它通过了对应 Gateway API bundle 版本附带的一致性测试。

实现必须在**无跳过测试（no skipped tests）**前提下通过 conformance 才算 conformant。开发阶段可以跳过，但正式宣称兼容的版本必须不跳过。

Extended 特性可按 Extended 契约进行关闭。

Gateway API conformance 是**版本相关**的：通过版本 N 的实现，不保证无需改动就能通过 N+1。

建议实现方向 Gateway API GitHub 仓库提交 conformance 报告，报告会包含所支持的 Gateway API 版本信息。

#### 版本兼容性

从 v1.0 起，支持 Gateway 与 GatewayClass 的实现必须设置新 Condition：`SupportedVersion`。

- `status: true` 表示当前安装的 CRD 版本被支持
- `status: false` 表示不被支持

### 标准状态字段与 Conditions

Gateway API 资源很多，但状态表达尽量保持一致：统一使用 Condition 类型和 `status.conditions` 字段。

大多数资源有 `status.conditions`，也有一些资源在命名空间化字段中嵌套 `conditions`。

例如：

- Gateway 的 `status.listeners`：每个 Listener 都有自己的 Conditions
- Route 的 `status.parents`：每个 parent（实现/归属链路）都有自己的 Conditions

Route 需要这样设计，是因为同一个 Route 可以附着到多个 Gateway，而这些 Gateway 可能由不同实现进行 reconcile。

常见 Condition 含义：

- `Accepted`：资源（或其中一部分）包含可接受配置，足以在实现控制的数据面产生某种效果。并不意味着“全部配置都有效”。
- `Programmed`：晚于 `Accepted` 的阶段，表示配置已被接受并已编程到数据面，预期在不久后可承载流量。它不表示“设置当下已立即就绪”，而是“将很快就绪”。
- `ResolvedRefs`：表示引用都合法，且引用对象存在并允许被引用。若为 `status: false`，表示至少一个引用无效，`message` 应说明哪些引用有问题。

实现者应查看各类型 godoc 中对这些 Condition 的精确定义。

此外，上游 `Conditions` 结构体包含可选字段 `observedGeneration`。实现**必须**使用该字段，并在生成状态时设置为对象的 `metadata.generation`，以便使用者判断状态是否对应对象的当前版本。

## TLS

TLS 在 Gateway API 中是一个持续扩展的大主题。用户视角可参考 [TLS Guide](https://gateway-api.sigs.k8s.io/guides/tls/)，本节补充实现者视角下的要点。

### Listener 隔离

在 Gateway 中，TLS 配置当前与 Listener 紧密绑定。为便于管理，建议实现朝“完整 Listener Isolation”目标演进。

建议行为：一次请求应最多匹配一个 Listener。比如定义了 `foo.example.com` 与 `*.example.com` 两个 Listener 时，请求 `foo.example.com` 应只通过前者路由，而不是同时进入通配 Listener。

不支持 Listener Isolation 的实现必须清晰文档化说明。未来计划引入 HTTPS Listener Isolation conformance 测试，确保声明支持该能力的实现行为一致。

### 间接配置

TLS 证书并不总是由 Gateway 拥有者直接管理。下列是常见模式：

#### 1. 来自外部系统的证书

部分厂商支持在 Kubernetes 外部托管证书。实现若可对接这些外部系统，可通过 Listener 的 TLS option 暴露能力。例如：

```yaml
listeners:
  - name: https
    protocol: HTTPS
    port: 443
    tls:
      mode: Terminate
      options:
        vendor.example.com/certificate-name: store-example-com
```

这里 `store-example-com` 指的是外部证书提供方中的证书名称。

#### 2. 自动生成并后续填充的 TLS 证书

很多用户希望证书自动生成。常见实现方式：额外控制器监听 Gateways/HTTPRoutes，自动签发证书并挂到 Gateway。

根据具体实现，可能要求 Gateway 拥有者通过 Gateway/Listener 级配置显式启用，比如在 `tls.options` 配置 `acme.io/cert-generator`。

这与 cert-manager 当前做法类似。历史上因为 v1.1 前要求必须填写 TLS CertificateRefs，cert-manager 需要先引用 Secret 再填充。v1.1 放宽校验后，创建时可不预填证书引用，自动化流程更简化。

#### 3. 由其他角色指定的证书

有些组织由应用开发者负责证书。可通过新增控制器和自定义 CRD，把 hostname 与用户提供证书关联，再将证书分发到匹配 hostname 的 Gateway Listener。这个模式通常也建议有 Listener/Gateway 级显式 opt-in。

### TLS 扩展的总体指导原则

在 Gateway API 上构建 TLS 扩展时，建议遵循：

1. 对任何 TLS option/annotation 使用带域名前缀且实现唯一的命名（如 `example.com/certificate-name`，不要只写 `certificate-name`）。
2. 不要在 option/annotation 值中直接编码敏感信息（如证书内容）。优先用简短、易理解的引用名称。虽然字段技术上可达 253 字符，建议控制在 50 字符以内以提升可读性和 UX。
3. 为支持扩展，Gateway API v1.1+ 不再强制 Listener 必填 TLS 配置。若 Listener TLS 配置不足，实现必须将该 Listener 的 `Programmed` 置为 `False`，reason 为 `InvalidTLSConfig`。
4. 无论支持哪些扩展，都必须支持 Gateway API 里可移植的核心 TLS 能力。扩展不能替代核心能力。

## 资源细节

对每个 conformance profile，都会定义一组实现应当 reconcile 的资源。以下按对象说明预期行为。

### GatewayClass

GatewayClass 的核心 `spec` 字段是 `controllerName`。每个实现应声明一个带域名前缀的唯一值（如 `example.com/example-ingress`）。

实现必须 watch **所有** GatewayClass，并 reconcile `controllerName` 匹配的对象。

对于匹配 `controllerName` 的 GatewayClass：

- 实现需要从中选择至少一个“兼容的” GatewayClass，并将其 `Accepted` Condition 设为 `status: true` 表示接受处理。
- 对于匹配但未被接受的 GatewayClass，`Accepted` 必须设为 `status: false`。

如果实现只能 reconcile 一个 GatewayClass，可只选择一个；如果支持多个，可选择任意多个。

若 GatewayClass 存在不兼容内容（例如引用了实现不支持的 `paramsRef`），实现应将其标记为未接受（`Accepted=False`）。

### Gateway

Gateway 要被实现 reconcile，必须在 `spec.gatewayClassName` 引用一个“存在且被实现接受（Accepted）”的 GatewayClass。

如果 Gateway 失去 reconcile 范围（例如引用的 GatewayClass 被删），实现可以在删除流程中移除其 status，但这不是强制要求。

### Routes

所有 Route 共享的规则：

- 必须附着到实现范围内（in-scope）的 parent，才能被视为可 reconcile。
- 实现必须通过具名 `parents` 字段更新每个 in-scope Route 的状态 Conditions。通常包括 `Accepted`、`Programmed`、`ResolvedRefs`。
- 对于已脱离范围的 Route，不应继续更新其 status，以免覆盖新 owner 的状态；`observedGeneration` 会帮助识别残留状态是否过期。

#### HTTPRoute

HTTPRoute 处理可检查的“明文 HTTP 流量”，包括在 Gateway 终止 TLS 后解密得到的 HTTP 流量，因此可按 path、method、headers 等 HTTP 属性路由。

#### TLSRoute

TLSRoute 在**不解密流量**的前提下，通过 SNI 对加密 TLS 流量进行路由到后端。

#### TCPRoute

TCPRoute 将进入某 Listener 的 TCP 流转发到指定后端之一。

#### UDPRoute

UDPRoute 将进入某 Listener 的 UDP 包转发到指定后端。

### ReferenceGrant

ReferenceGrant 是跨命名空间引用授权资源：由“被引用对象所在命名空间”的拥有者创建，用于有选择地允许其他命名空间的 Gateway API 对象引用本命名空间对象。

它创建在“被授权访问对象”的同一命名空间，可按来源命名空间、来源 Kind（或两者）做约束。

支持跨命名空间引用的实现必须 watch ReferenceGrant，并 reconcile 所有“指向 in-scope Gateway API 对象所引用目标”的 ReferenceGrant。
