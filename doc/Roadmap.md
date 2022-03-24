## Roadmap
### before release
- [x] very simple http1 spa server
- [x] very simple admin server(http api)
- [x] ssl(including wildcard domain ssl)
- [x] docker release
- [x] simple usage doc

### version 1.0.x
- [x] 80 and 443 both support
- [ ] ~~compression~~ done @ v1.2.0.
- [ ] ~~multiple tls support~~ the feature may do not need.
- [x] cache file(cache all files in memory without LRU)

### version 1.1.x
- [x] more doc(how to update static files)
- [x] rewrite Dockerfile to reduce docker image size
- [x] cache improve(big file ignore config option and if-range header support)
- [x] header`cache-control` for client cache
- [ ] ~~header `etag` for client cache~~ [warp #462](https://github.com/seanmonstar/warp/issues/462)
- [x] 80 redirect to 443 config option
- [x] compression for js/icon/json/css/html (only support gzip algo, only compress cached files, and ~~will occur error when client don't support gzip~~(fix @ v1.2))

### version 1.2.x
- [x] more log for debug and trace
- [x] basic CORS
- [x] compress regression support(if client don't send accept-encoding header(including gzip), will send back data from file instead of cache)
- [x] hot reload web static server(use SO_REUSEPORT *nix api, so it may be wrong with Windows).
- [ ] ~~different config(cors/cache strategy/https and so on) for different domain.~~ (if this is needed?)

### version 1.3.x
- [ ] cache File `Range` Header support
- [ ] drop self maintained `Warp`(copy out needed code from Warp)
- [ ] `HEAD` request support or drop
