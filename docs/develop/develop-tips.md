# Develop Tips

## Docker Build

Use [moby/buildkit](https://github.com/moby/buildkit) to speed up building process.

```bash
VERSION=1.2.1

DOCKER_BUILDKIT=1 docker build . -t="ghcr.io/fornetcode/spa-server:$VERSION"

```

## SSL self sign

ref: https://docs.rancher.cn/docs/rancher2/installation/resources/advanced/self-signed-ssl/_index/

## Test Let's Encrypt with Pebble

config should be set as follows:

```shell
# I try to change Pebble httpPort, but does not success. so the port must be Pebble default port.
http.port = 8080
https.acme {
      emails = ["mailto:zsy.evan@gmail.com"]
      # directory to store account and certificate
      # optional, default is ${file_dir}/acme
      #dir = "/data/acme"
      type = ci
      # this is for Pebble CA, Http needs this to connect Pebble with Https
      ci_ca_path = "./tests/data/pebble/certs/pebble.minica.pem"
  }
```

**Remember to remove `tests/data/web/acme` directory when Pebble reinit.**

## reqwest

when reqwest use rustls, redirect would have problems: it would not redirect event with Policy::default().