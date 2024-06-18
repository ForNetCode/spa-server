# Break Changes
## V2.2.2(2024-06-18)
* spa-server: `http_redirect_to_https` convert from bool to u32.
## V2.2.0(2024-06-15)
* spa-server: `port, addr` => `http.port, http.addr`
## v2.0.0(2024-04-02)
* spa-server: `file/upload` param change for easy write. 

## v1.2.6(2022-05-11)
* spa-server: move config path in docker: `config.conf` => `config/cnfig.conf` 

## v1.2.5(2022-04-06)
* spa-server config breaks for support multiple domain config.
### Migrate(Change Config Position)
* `compress` => `cache.compress`
* `https.private` => `https.ssl.private`
* `https.public` => `https.ssl.public`

## v1.2.3(2022-04-01)
* change admin server http api "upload/path" to "upload/position", and add a `format` option.
