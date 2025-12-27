# Kind 集群下的 Gateway API 测试环境

用于在本地 kind 集群里验证 Switchboard 的 Gateway API 集成。默认资源都放在 `default` namespace，与 `K8sGatewayResourceConfig.gateway_namespace` 默认值保持一致。

## 目录说明
- `kind-config.yaml`：kind 集群定义，暴露宿主机 80/443 端口到控制平面节点。
- `kustomization.yaml`：一键应用所有测试用的 Gateway/Route/Service/TLS 资源。
- `manifests/`：示例 GatewayClass、Gateway、HTTPRoute、后端服务和 TLS Secret。
- `setup-kind.sh`：创建集群并安装 Gateway API CRDs 的快速脚本。

## 准备
- 安装 [kind](https://kind.sigs.k8s.io/) (>= 0.22)
- 安装 `kubectl` (推荐与集群同版本)
- 本地 Docker/Podman 能拉取镜像

## 快速开始
```bash
# 1) 创建 kind 集群
kind create cluster --name switchboard-gateway --config tests/k8s/kind-config.yaml

# 2) 安装 Gateway API CRDs（标准安装包）
kubectl create namespace gateway-system --dry-run=client -o yaml | kubectl apply -f -
kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.4.1/standard-install.yaml
kubectl wait --for=condition=Available --timeout=180s -n gateway-system deploy/gateway-api-admission-server

# 3) 应用测试资源（GatewayClass/Gateway/HTTPRoute/后端/TLS）
kubectl apply -k tests/k8s

# 4) 查看资源
kubectl get gatewayclass,gateway,httproute -A
kubectl get svc,deploy -n default
```

## 如何验证
- 依赖的数据面（Switchboard Kernel）启动后，直接在宿主机访问：
  ```bash
  curl -H "Host: echo.localtest.me" http://127.0.0.1:80/
  ```
  如果启用 TLS Listener，也可以：
  ```bash
  curl -k -H "Host: echo.localtest.me" https://127.0.0.1:443/
  ```
- 如果只想验证后端 Pod 是否正常，可端口转发 Service：
  ```bash
  kubectl port-forward svc/echo 18080:80
  curl http://127.0.0.1:18080/
  ```

## 清理
```bash
kind delete cluster --name switchboard-gateway
```

## 备注
- GatewayClass 的 `controllerName` 已设置为 `switchboard.io/gateway-controller`，与代码中的常量保持一致。
- 所有 HTTPRoute/Gateway/Secret 在 `default` 命名空间，如果修改了 `gateway_namespace`，请同时调整 YAML 中的 `namespace`。
- 默认证书是自签发的 `echo.localtest.me`，仅用于本地测试。
