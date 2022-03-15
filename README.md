## SPA-SERVER [WIP]
This project is to create a http server which make deploy and manage multiple single page applications easy.

The more details are described at [SPA 发布辅助工具](https://github.com/timzaak/blog/issues/80) in Chinese.


### run code
```
git submodule init && git submodule update
RUST_LOG=info cargo run bin/main.rs
```

### why use self maintained warp
[#171 Add reply::file(path) helper](https://github.com/seanmonstar/warp/issues/171)

will change back until this issue can be solved

### Roadmap 
#### before release
- [x] very simple http1 spa server
- [x] very simple admin server(http api)
- [x] single tls (support http://cookcode.cc/selfsign self sign, others does not test now)
- [x] docker release(simple)

#### version 1.x
- [ ] 80 and 443 both support
- [ ] compression
- [ ] http2
- [ ] multiple tls(need to replace warp by hyper to support)
- [ ] domain visit count/data analysis
- [ ] make server more fast
- [ ] s3 storage(just copy files to local file system?)
- [ ] memory fs(and reimplement s3 storage, do not need to copy files to local file system)
