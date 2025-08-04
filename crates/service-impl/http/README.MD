next level gateway
# switchboard http, aka **sbh**
## update controled by sbh supervisor
```text,ignore
sbh supervisor -> [config event] -> rebuild new service --+--> 
                                                          |
service running                  ->                  --ArcSwap -> new service running
                                                        \
                                                         +------> drop when all the request finished
```

## managed controled by sb
```text,ignore
sb -> delete -+-->
              |
sbh service --v-- Shutdown ---> Clean up  
```

```sh
sb create h --name "gateway" --config "etc/sb/h/gateway.toml"
sb bind [::]:80 @gateway
```

## config
```toml
[layer.timeout]
class = "timeout"
timeout = "5s"

[service.frontend-svc]
class = "client"
layers = ["timeout"]
host = "192.168.15.15:80", 
schema = "http"

[service.api-svc]
class = "client"
layers = ["timeout"]
host = "192.168.16.92:8080", 
schema = "http"

[router.match]
class = "match"
matchs = { "/api/"= "api" }

[router.match.routes]
api = "api-svc"
"[fallback]" = "frontend-svc"

```


```toml
[function.gateway]
class = "gateway"
timeout = "5s"
[function.gateway.match]
"/api/" = "http://192.168.16.92:8080/"
fallback = "http://192.168.15.15:80/"
```


```typescript
const frontend = service("frontend", endpoint("http://frontend:80/"));
const api = service("api", endpoint("http://api:8080/"));
const router = router(
    "router", 
    {
        "api": api,
        "frontend": frontend
    },
    match({
        "/api/": "api",
        "$fallback": "frontend"
    })
);

export default [
    ...layers,
    ...services,
    ...routers,
    ...compositions,
]
```
# SBH设计
timeout Layer

0/1 => incoming
1/0 => service
1/1 => layer
1/n => router
n/1 => hub
m/n => bundle

原则1： 哪来的回哪去
原则2： 扁平化
```rust
async fn call(req: Request, source: PortKey, out: Out) -> Response {

}
```
