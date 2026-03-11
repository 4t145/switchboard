# 当前 Kubernetes Gateway 解析实现的不足

本文整理 `controller` 目前从 Kubernetes Gateway API 构建配置时的主要语义缺口，方便后续迭代时按优先级补齐。

## 1. ParentRef 匹配条件过于宽松

当前逻辑主要通过 `HTTPRoute.parentRefs[].name` 关联到 `Gateway`。这在语义上不完整，理想实现应同时考虑以下字段及默认值规则：

- `group`
- `kind`
- `namespace`
- `sectionName`

否则可能出现“同名但非目标 Gateway”被误绑定的情况。

## 2. 未严格处理 listener 级路由绑定约束

Gateway API 的 listener 可通过 `allowedRoutes` 限制允许绑定的 Route 类型与命名空间范围。当前实现未完整校验这些约束，可能把本应被拒绝的 Route 纳入配置。

## 3. 同名 Gateway 跨命名空间冲突风险

当不同 namespace 下存在同名 Gateway 时，如果内部映射键只使用 `name`，会有覆盖或串联风险。理想做法应使用 `namespace/name` 作为唯一键。

## 4. HTTPRoute 扫描范围受限于单一 namespace

当前实现基于单个 `gateway_namespace` 拉取 HTTPRoute。对于跨 namespace 绑定场景（尤其是平台化部署）支持不足。后续可考虑：

- 支持多 namespace 输入
- 或根据 `allowedRoutes` + namespace selector 动态判定可见范围

## 5. TLS 证书引用 namespace 语义需更严格

当 `certificateRef.namespace` 为空时，理想默认应与 Gateway 所在 namespace 对齐。当前实现的默认 namespaced client 语义需要再次核对，以避免误取错误 namespace 的 Secret。

## 6. 缺少基于状态条件的可用性过滤

如需输出“可部署配置”，建议根据状态条件（例如 `Accepted` / `ResolvedRefs`）过滤未生效对象。当前实现更接近“声明态聚合”，而非“已就绪态聚合”。

## 7. 可观测性与错误归因信息不足

目前出错时更偏向整体失败。建议补充：

- 对单个 Gateway/Route 的细粒度告警
- 不可解析对象的原因列表
- 结构化统计（成功/跳过/失败数）

这会显著提升排障效率。

## 结论

当前实现可以支撑基础场景，但离“严格遵循 Gateway API 语义”仍有差距。建议优先处理 ParentRef 精确匹配、listener 约束、唯一键冲突与状态条件过滤四个方向。
