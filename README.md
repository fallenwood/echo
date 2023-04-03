Echo
---

[![Build Status](https://fallenwood.visualstudio.com/Public%20Projects/_apis/build/status%2Ffallenwood.echo?branchName=master)](https://fallenwood.visualstudio.com/Public%20Projects/_build/latest?definitionId=18&branchName=master)

A simple *http echo* server written in rust

## Building

### Build directly

```
cargo build
```

### Docker

```
docker build .
```

## Testing

```
→ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/echo`
```

```
→ curl -v http://127.0.0.1:3000?delay=1200
*   Trying 127.0.0.1:3000...
* Connected to 127.0.0.1 (127.0.0.1) port 3000 (#0)
> GET /?delay=1200 HTTP/1.1
> Host: 127.0.0.1:3000
> User-Agent: curl/7.85.0
> Accept: */*
>
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< content-type: text/plain; charset=utf-8
< x-client-ip: 127.0.0.1
< x-request-id: dae3cbb9-b505-4a9f-a222-7b35f504c7ba
< x-response-time: 1202
< content-length: 0
< date: Mon, 03 Apr 2023 15:51:25 GMT
<
* Connection #0 to host 127.0.0.1 left intact
```

## TODOs

- [] Tracing
- [] Support other methodes
    - [] POST

## LICENSE
BSD
