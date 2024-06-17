# HTTPS

## 配置

全局默认配置 `https.ssl`，主要用于泛域名或整个服务都是一个域名，（支持数组）
全局Let's Encrypt配置：`https.acme`，用于 acme 全局签名。 基于 TLS-ALPN-01 做认证，不支持通配符域名，每个主机名使用单独的证书。
域名单独配置(DomainHttpsConfig): `domains[].https`。 域名单独配置不支持 acme， 直接放到全局上跑。

### 配置生效优先级：

1. 域名单独配置: `domains[].https.disabled`(禁用 https)
2. 域名单独配置：`domains[].https.ssl`
3. 全局Let's Encrypt配置：`https.acme` 和 全局默认配置 `https.ssl`相互互斥，两者不可同时存在。 (简单做)

## ACME 引发的问题

### 1. ACME 加载时间问题

ACME 不是配置完立马生效。需要一定时间，这段时间怎么办？

直接 reject，并打印日志

### 2. 域名新增、删除问题

做的简单一些。 `https.acme.domains` 配置 acme 域名列表。 这样就没有删除问题。
域名新增走 hotReload。