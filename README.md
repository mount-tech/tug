# tug

Minimalist easy to configure web server. Alpha State.


Installation:
```
cargo install
```

Usage:
- Auto-detect tug.toml in currently direction:
```
tug
```
- Explicitly define config location:
```
tug some_config.toml
```

Example config:
```
log = "test.log"

[[server]]
host = "127.0.0.1:7357"
gzip = false
root = "./src"
markdown = { js = "/js/test.js", css = "/css/test.css" }
```

Current directives:
- host - host name/ip to serve at, default = `"127.0.0.1:8080"`
- root - root folder to serve, default - `"./"`
- gzip - gzip responses, default - `true`
- log - define a path for the (currently) global config, default - `output.log`
- markdown - defines whether to render any markdown files in root, default - `None`
