# Break Changes

## v1.2.5(2022-04-06)
* spa-server config breaks for support multiple domain config.
### Migrate(Change Config Position)
* `compress` => `cache.compress`
* `https.private` => `https.ssl.private`
* `https.public` => `https.ssl.public`

## v1.2.3(2022-04-01)
* change admin server http api "upload/path" to "upload/position", and add a `format` option.
