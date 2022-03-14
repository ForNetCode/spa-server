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
#### version 0.x
- [x] very simple http1 spa server

#### version 1.x
- [ ] client upload file and control server 
- [ ] tls (wildcard domain firstly)
- [ ] docker release
- [ ] S3 storage(just copy s3 files to local file system)

#### version 2.x
- [ ] cache(and reimplement s3 storage, do not need to copy files to local file system)
- [ ] compression
- [ ] http2

#### version 3.x
- [ ] make server more fast
- [ ] domain visit count/data analysis
