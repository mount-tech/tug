# tug

ALPHA easy configurable web server.

Usage:
```
tug tug.toml
```

Example config:
```
[[server]]
host = "127.0.0.1:7357"
gzip = false
root = "./src"
```

Current directives:
- host - host name/ip to serve at
- root - root folder to serve
- gzip - gzip responses
