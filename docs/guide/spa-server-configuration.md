# Configuration
## Overview
The config format is [HOCON(Human-Optimized Config Object Notation)](https://github.com/lightbend/config/blob/main/HOCON.md).

The config default path is './config.conf', you can change it by environment `SPA_CONFIG`.
## Config Reference


```hocon
# http bind, if set port <= 0, will disable http server(need set https config)
port = 80
addr = "0.0.0.0"

# directory to store static web files. if you use docker, please mount a persistence volume for it.
file_dir = "/data"

# enable cors, default is false, its implementation is simple now.
# Access-Control-Allow-Origin: *
# Access-Control-Allow-Methods: OPTION,GET,HEAD
# Access-Control-Max-Age: 3600
// cors = true

# https config, optional
//https {
//  # private ssl key
//  private = "private.key path",
//  # public ssl cert
//  public = "public.cert path"

//  # https bind address
//  port = 443
//  addr = "0.0.0.0"

//  # if set true, http server(80) will send client
//  # status code:301(Moved Permanently) to tell client redirect to https
//  # optional, default is false
//  http_redirect_to_https = false

//  # gzip compression for js/json/icon/json, default is false,
//  # only support gzip algo, and only compress cached files,
//  # be careful to set it true
//  compression = false

//}

# cache config
//cache {
//  # if file size > max_size, it will not be cached. default is 10485760 (10MB).
//  # do not use size unit format like 50MB!
//  # It's blocked by [hocon #62](https://github.com/mockersf/hocon.rs/issues/62)
//  max_size = 10485760  //10MB 10*1024*1024

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
//}

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
//  # default is 30MB(30*1024*1024)
//  max_upload_size = 31457280
//}
```