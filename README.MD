Noto is an unified gateway for every traffic.

```
"localhost:8888" => "noto://tcp/socks5/my-s5-proxy"
```

```sh
noto bind [::]:10808 socks5
noto bind [::]:10808 tf/127.0.0.1:10809
noto unbind [::]:10808
noto bind [::]:10808 @my-static-blog
noto list
noto reload
```