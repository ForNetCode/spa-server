# JS-APP-EXAMPLE

This is an example of React frontend project. The usage of `spa-client` is simple.

## How to run this example

start a spa-server with admin-server enabled, change check `.env` config if is right, run the following shell to
upload files to spa-server.

```shell
npm install && npm run build && npm run upload && npm run release
```

PS: `local.fornetcode.com` is routed to 127.0.0.1

There has a nice [document](https://fornetcode.github.io/spa-server/) deployed by GitHub Pages, You can get more
information here.

### Test bash

```shell
# multiple
npm run upload:m1 && npm run release:m1
npm run upload:m2 && npm run release:m2
```
