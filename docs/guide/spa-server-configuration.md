# Configuration

## Overview

The config format toml.

The config default path is `./config.toml`, you can change it by environment `SPA_CONFIG`.

## Config Reference

## Toml Format Config

```toml
## directory to store static web files. if you use docker, please mount a persistence volume for it.
file_dir = "./data"

## enable cors, default is none, if cors is [], then all cors is ok.
## Access-Control-Allow-Origin: $ORIGIN
## Access-Control-Allow-Methods: OPTION,GET,HEAD
## Access-Control-Max-Age: 3600
## If you put the server behind HTTPS proxy, please enable it, or domains.cors = ['http://www.example.com:8080']
## Attension: domains.cors config would overwrite the cors config, rather than merge this.
cors = []
## http bind, if set port <= 0 or remove http, will disable http server(need set https config)
[http]
port = 80
addr = "0.0.0.0"
## port when serving public network,default is http port. external_port should not be 0.
# external_port = 80

## optional, when https enabled, redirect_https default value true
## it would the port would be https.external_port(https.external_port should be defined), otherwise is false
# redirect_https = true

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
# cors = ['https://www.example.com', 'http://www.baidu.com']
# [domains.https]
## optional, when https enabled, redirect_https default value true
## it would the port would be https.external_port(https.external_port should be defined), otherwise is false
# redirect_https = 443

```
