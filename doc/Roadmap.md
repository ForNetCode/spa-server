# Roadmap
### Version 1.3.x
Now there is no real roadmap for v1.3.x, need users.

### near future before v1.3.0
- [ ] release trigger by tag
- [ ] add wix configs, and release window msi for spa-client(need window pc to do this)
- [ ] test integrate, and add ci to run it

### Version 1.2.5
- [x] add docker image cache for (spa-client|spa-server)-docker-cd.yml to speed cd process
- [ ] use vuepress to rebuild docs, ready to get the world known it


### Version 1.2.4(client:v0.1.1)
- [x] release commandline of spa-client for mac/ios/linux (by GitHub Actions), put them with GitHub release page
- [x] fix possible bugs about uploading and spa-client(-js)
- [x] build: release docker image by GitHubActions
- [x] build: add docker image for spa-client
- [x] doc: how to use spa-client image
- [x] improve: add debug log for spa-server request

### version 1.2.3(client:v0.1.0)
- [x] admin server export http api to accept files to local file system
- [x] add client to sync local files to admin server（retry support）
- [ ] ~~release server/client to crate~~ [crate needs dep version, need replace warp firstly](https://github.com/rust-lang/cargo/issues/1565)
- [x] doc about how to use with shell client
#### add js plugin
- [x] add js wrapper for spa-client
- [x] and example/test frontend repo
- [x] doc about how to use with js client
- [x] release js wrapper to npm.org

### version 1.2.2
- [x] cache File `Range` Header support
- [ ] ~~drop self maintained `Warp`(copy out needed code from Warp)~~ (so much code from warp/fs, I give up after try, will wait Warp release proper version)
- [x] `HEAD` request support or drop(support, don't need to do anything)

### version 1.2.1
- [x] more log for debug and trace
- [x] basic CORS
- [x] compress regression support(~~if client don't send accept-encoding header(including gzip), will send back data from file instead of cache~~ improved by v1.2.3)
- [x] hot reload web static server(use SO_REUSEPORT *nix api, so it may be wrong with Windows).
- [ ] ~~different config(cors/cache strategy/https and so on) for different domain.~~ (if this is needed?)

### version 1.1.x
- [x] more doc(how to update static files)
- [x] rewrite Dockerfile to reduce docker image size
- [x] cache improve(big file ignore config option and if-range header support)
- [x] header`cache-control` for client cache
- [ ] ~~header `etag` for client cache~~ [warp #462](https://github.com/seanmonstar/warp/issues/462)
- [x] 80 redirect to 443 config option
- [x] compression for js/icon/json/css/html (only support gzip algo, only compress cached files, and ~~will occur error when client don't support gzip~~(fix @ v1.2))

### version 1.0.x
- [x] 80 and 443 both support
- [ ] ~~compression~~ done @ v1.2.0.
- [ ] ~~multiple tls support~~ the feature may do not need.
- [x] cache file(cache all files in memory without LRU)

### before release
- [x] very simple http1 spa server
- [x] very simple admin server(http api)
- [x] ssl(including wildcard domain ssl)
- [x] docker release
- [x] simple usage doc
