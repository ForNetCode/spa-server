# SPA-Client
This is js wrapper for spa-client.
More documents can find [here](https://timzaak.github.io/spa-server/guide/spa-client-npm-package.html)


### Operating Systems

|                  | node12 | node14 | node16 |
| ---------------- |--------|--------|--------|
| Windows x64      | ✓      | ✓      | ✓      |
| Windows x32      | ✓      | ✓      | ✓      |
| Windows arm64    | x      | x      | x      |
| macOS x64        | ✓      | ✓      | ✓      |
| macOS arm64      | ✓      | ✓      | ✓      |
| Linux x64 gnu    | x      | x      | ✓      |
| Linux x64 musl   | ✓      | ✓      | ✓      |
| Linux arm gnu    | ✓      | ✓      | ✓      |
| Linux arm64 gnu  | ✓      | ✓      | ✓      |
| Linux arm64 musl | ✓      | ✓      | ✓      |
| Android arm64    | ✓      | ✓      | ✓      |
| Android armv7    | ✓      | ✓      | ✓      |
| FreeBSD x64      | ✓      | ✓      | ✓      |

Windows arm64: https://github.com/briansmith/ring/issues/1167

Linux x64 gnu(node12,node14): Error: /build/jsclient/spa-client.linux-x64-gnu.node: cannot allocate memory in static TLS block, But I test on my OpenSUSE linux(x64 gnu) with nodeV14.17.1 successfully.
