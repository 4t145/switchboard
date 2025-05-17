switchboard is an unified gateway for every traffic.

```
"localhost:8888" => "sb://tcp/socks5/my-s5-proxy"
```

```sh
sb bind [::]:10808 socks5
sb bind [::]:10808 tf/127.0.0.1:10809
sb unbind [::]:10808
sb bind [::]:10808 @my-static-blog
sb list
sb reload
```