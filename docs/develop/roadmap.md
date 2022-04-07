# Roadmap
### Version 1.3.x

Now there is no real roadmap for v1.3.x, need users.

### near future before v1.3.0
- [ ] CD: release triggered by new tag
- [ ] CD: add wix configs, and release window msi package for spa-client(need window pc to do this)
- [ ] TEST: test integrate, and add ci to run it
- [ ] Feat: S3 Support

### Version 1.2.5(client v0.1.2)
- [x] build: add docker image cache for (spa-client|spa-server)-docker-cd.yml to speed cd process
- [x] doc: use VitePress to rebuild docs, ready to get the world known it
- [x] build: add CD for doc release
- [x] feat: support multiple config for different domain (break change for config file)
- [x] feat: support multiple ssl
- [ ] ~~fix: disable put online domain which does not have correct ssl in server when https opened.~~(need to confirm if it's a bug?)
- [x] doc: multiple config doc
- [x] fix: fix wrong check when release new domain
- [ ] client fix: npm package error 
