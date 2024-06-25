# Configuration

## Overview

The config format
is [HOCON(Human-Optimized Config Object Notation)](https://github.com/lightbend/config/blob/main/HOCON.md).

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

# [https]
# port = 443 # https bind address
# addr = "0.0.0.0"
## if set true, http server(80) will send client
## status code:301(Moved Permanently) to tell client redirect to https
## optional, default is false
# http_redirect_to_https = false

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
# cors = false
# [domains.https]
# http_redirect_to_https = 443
## this would be usefully when set https.acme
# disable_acme = false
# [domains.https.ssl]

# [domains.cache]
# max_size = 10_000_000
# compression = false
# [[domains.cache.client_cache]]
# expire = '30d' # 30day
# extension_names = ['icon', 'gif', 'jpg', 'jpeg', 'png', 'js']

```

## Hocon Format Config

**Attention: hocon format would not support in the future.**

```hocon
# http bind, if set port <= 0 or remove http, will disable http server(need set https config)
http {
  port = 80
  addr = "0.0.0.0"
}

# directory to store static web files. if you use docker, please mount a persistence volume for it.
file_dir = "/data"

# enable cors, default is false, its implementation is simple now.
# Access-Control-Allow-Origin: $ORIGIN
# Access-Control-Allow-Methods: OPTION,GET,HEAD
# Access-Control-Max-Age: 3600
// cors = true
# https config, optional
//https {
//  # default value for https ssl
//  ssl {
//    # private ssl key
//    private = "private.key path",
//    # public ssl cert
//    public = "public.cert path"
//  }
//  # acme config, it doest not support run with https.ssl config.
//  acme {
//    # emails to Let's Encrypt needs to interact.
//    emails = ["mailto:email@example.com"]
//    # directory to store account and certificate
//    # optional, default is ${file_dir}/acme
//    // dir = "/data/acme"
//    # ci / stage / prod, default is prod, ci is just for CI test with Pebble, don't use it.
//    //type = prod
//  }

//  # https bind address
//  port = 443
//  addr = "0.0.0.0"

//  # if set true, http server(80) will send client
//  # status code:301(Moved Permanently) to tell client redirect to https
//  # optional, default is null
//  http_redirect_to_https = 443
//}

# default cache config
cache {
//  # if file size > max_size, it will not be cached. default is (10MB).
    max_size = 10MB
//  # http header Cache-Control config,
//  # optional, if not set, won't sender this header to client
//  client_cache = [{
//    expire = 30d
//    extension_names = [icon,gif,jpg,jpeg,png,js]
//  }, {
//    // set 0, would set Cache-Control: no-cache
//    expire = 0
//    extension_names = [html]
//  }]
//  # gzip compression for js/json/icon/json, default is false,
//  # only support gzip algo, and only compress cached files,
//  # be careful to set it true
//  compression = false

}

//# admin server config
//# admin server don't support hot reload. the config should not change.
//# optional, and it's disabled by default.
//# if you use spa-client to upload files, control version. Need to open it
//admin_config {
//# bind host
//  port = 9000
//  addr = "127.0.0.1"
//  # this is used to check client request
//  # put it in http header,  Authorization: Bearer $token
//  token = "token"
//  # max file size allowed to be uploaded,
//  # default is 30MB(30*1000*1000)
//  max_upload_size = 30*1000*1000
//  # delete deprecated version by cron
//  deprecated_version_delete {
//    # default value: every day at 3am.
//    cron: "0 0 3 * * *",
//    # default value is 2
//    max_preserve: 2,
//  }
//}


# optional, domains specfic config, it will use the default config if not set
//domains = [{
//  # domain name
//  domain: "www.example.com",
//  // optional, same with cache config, if not set, will use default cache config.
//  cache: {
//    client_cache:${cache.client_cache}
//    max_size: ${cache.max_size}
//  },
//  # cors
//  cors: ${cors},
//  # domain https config, if not set, will use default https config.
//  https: {
//    ssl: ${https.ssl}
//    http_redirect_to_https: ${https.http_redirect_to_https}
//  }
//}]
```