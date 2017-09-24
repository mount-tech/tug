# tug

ALPHA easy configurable web server.


Installation:
```
cargo install
```

Usage:
- Auto-detect tug.toml in currently direction
```
tug
```
- Explicitly define config location:
```
tug some_config.toml
```

Example config:
```
[[server]]
host = "127.0.0.1:7357"
gzip = false
root = "./src"
```

Current directives:
- host - host name/ip to serve at, default = "127.0.0.1:8080"
- root - root folder to serve, default - "./"
- gzip - gzip responses, default - true
