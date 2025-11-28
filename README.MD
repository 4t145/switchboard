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

# 编辑页面
对于不同的服务，应当提供对应的配置编辑器


# 权限
todo

# 版本控制
todo