# Configuration

## Overview

You can set config file path by `config-dir` commandline option or by environment variables `SPA_CLIENT_CONFIG`,
you can also set all config by environment variables like
[.env](https://github.com/fornetcode/spa-server/blob/master/example/js-app-example/.env). Config override order is
command line option > config file > environment.

## Reference

## Toml Format Config

```toml
[server]
address = "http://127.0.0.1:9000"
auth_token = "token"
# [upload]
## default value is:3
# parallel = 3
```