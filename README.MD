# **SWITCHBOARD**
High-performance Easy-to-use Ergonomics Next-generation Post-modern Cross-platform AI-powered Super (And any irresponsible adjective) Gateway 

```
"localhost:8888" => "sb://tcp/socks5/my-s5-proxy"
```

```sh
sb bind [::]:10808 socks5
sb bind [::]:10808 tf/127.0.0.1:10809
sb bind [::]:9096 uds//run/fc.socket
sb unbind [::]:10808
sb bind [::]:10808 @my-static-blog
sb bind [::]:80 h
sb list
sb reload
```

sb => sb-kernel

# switchboard-http
```sh
sbh forward /session [::1]:8080
```
满足网络配置和路由

## 后端
### k8s/docker
### 静态文件
### 网络服务

## 插件


# 可观测性
## 日志（log）
## 度量（metric）
## 追踪（tracing）

# 配置
配置可以有多个后端
1. 出于现代性的考虑，配置必须是可以由脚本计算产生。
2. 配置中的关键资源应当可以从环境/网络等其它地方获取。
## 配置的热更新策略
应当关注每一份配置项是否有变动
1. 如果某个服务本身配置变动，重新构建该服务。

# 遥控器（Controller）
一个挂件，负责在所有数据源，前端，网关之间传输数据。
1. 扫描可配置网络，提供给前端配置项
2. 接收配置变更信息，重新渲染配置，通知所有网关
3. 扫描所有网关状态，通知给前端
4. 提供api，供外部使用
    - http api，给前端/其它应用使用
    - local socket api，给本地sbc ctl使用

可以和所有网关实例通讯
可以给前端提供信息

# 前端
1. 对于不同的服务，应当提供对应的配置编辑器
2. 

# 拓展
## v8引擎


# 权限


# 版本控制
