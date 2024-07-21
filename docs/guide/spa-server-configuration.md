# Configuration

## Overview

The config format toml.

The config default path is './config.conf', you can change it by environment `SPA_CONFIG`.

## Config Reference

## Toml Format Config

```toml
## directory to store static web files. if you use docker, please mount a persistence volume for it.
file_dir = "/data"

## enable cors, default is false, its implementation is simple now.
## Access-Control-Allow-Origin: $ORIGIN
## Access-Control-Allow-Methods: OPTION,GET,HEAD
## Access-Control-Max-Age: 3600
cors = false
## http bind, if set port <= 0 or remove http, will disable http server(need set https config)
[http]
port = 80
addr = "0.0.0.0"
## port when serving public PI,default is http port. external_port should not be 0.
# external_port = 80

## optional, when https enabled, redirect_https default value true
## it would the port would be https.external_port(https.external_port should be defined), otherwise is false
# redirect_https = true

# [https]
# port = 443 # https bind address
# addr = "0.0.0.0"
## port when serving public PI,default is https port. external_port should not be 0.
# external_port = 443

## if set true, http server(80) will send client
## status code:301(Moved Permanently) to tell client redirect to https

## default value for https ssl
# [https.ssl]
# private = "private.key path" # private ssl key
# public = "public.cert path" # public ssl cert

## acme config, it doest not support run with https.ssl config.
# [https.acme]
## emails to Let's Encrypt needs to interact.
# emails = ["mailto:email@example.com"]

## directory to store account and certificate
## optional, default is ${file_dir}/acme
# dir = "/data/acme"

## ci / stage / prod, default is prod, ci is just for CI test with Pebble, don't use it.
# type = prod

## default cache config
[cache]
## if file size > max_size, it will not be cached. default is (10MB).
# max_size = 10_000_000
## gzip compression for js/json/icon/json, default is false,
## only support gzip algo, and only compress cached files,
## be careful to set it true
# compression = false

## http header Cache-Control config,
## optional, if not set, won't sender this header to client
# [[cache.client_cache]]
## 30day
# expire = '30d'
# extension_names = ['icon', 'gif', 'jpg', 'jpeg', 'png', 'js']
# [[cache.client_cache]]
# expire = '0'
# extension_names = ['html']

## admin server config
## admin server don't support hot reload. the config should not change.
## optional, and it's disabled by default.
## if you use spa-client to upload files, control version. Need to open it
# [admin_config]
# port = 9000
# addr = "127.0.0.1"

## this is used to check client request
## put it in http header,  Authorization: Bearer $token
# token = "token"
## max file size allowed to be uploaded,
## default is 30MB(30*1000*1000)
# max_upload_size = 30_1000_1000
## delete deprecated version by cron
# [admin_config.deprecated_version_delete]
## default value: every day at 3am.
# cron = "0 0 3 * * *"
## default value is 2
# max_preserve = 2

## optional, domains specfic config, it will use the default config if not set
# [[domains]]
# domain = "www.example.com"
## optional, `example.com` would redirect to `www.example.com`
# alias = ["example.com"]
# cors = false
# [domains.https]
## optional, when https enabled, redirect_https default value true
## it would the port would be https.external_port(https.external_port should be defined), otherwise is false
# redirect_https = 443

## this would be usefully when set https.acme
# disable_acme = false
# [domains.https.ssl]

# [domains.cache]
# max_size = 10_000_000
# compression = false
# [[domains.cache.client_cache]]
# expire = '30d' # 30day
# extension_names = ['icon', 'gif', 'jpg', 'jpeg', 'png', 'js']


# [openTelemetry]
# endpoint = "http://localhost:4317"
```
