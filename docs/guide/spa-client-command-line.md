# Command Line
## Overview
spa-client it a command line tool to help user upload files and release new SPA.

## Usage Example
There are some usage examples of `spa-client`, you also can get help by typing `spa-client -h`.
```shell
# upload static files to admin server, if not set $OPT_VERSION, will try to 
spa-client - $CONFIG_PATH upload $STATIC_FILE_PATH $DOMAIN $OPT_VERSION -p 3

# tell admin server to release the specific domain version to public. if don't set $OPT_VERSION, will fetch the max version to be online, if the max version it under uploading process, release will fail. 
spa-client - $CONFIG_PATH release $DOMAIN $OPT_VERSION

# get info of the specific domain or all domain, just like the admin server http api.
spa-client - $CONFIG_PATH info $OPT_DOMAIN

# reload spa-server, this is used to reload https cert
spa-client - $CONFIG_PATH reload
```
You could use `upload` and `relase` commands to release new version.

There also provide http api to interact with admin server,

```shell
# Uploading Files By scp and release 
scp $SPA_DIRECTORY \
user@ip:$(curl "http://$ADMIN_SERVER/upload/position?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN") &&\
curl "http://$ADMIN_SERVER/update_version?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN"
```

## Docker Image
We also release a docker image for spa-client.
```shell
$ docker run --rm -it -v $CONFIG_FILE_PATH:/client.conf \
 timzaak/client spa-client -c /client.conf info
```

## Binary package
You can get the packed binary in [Release Notes](https://github.com/timzaak/spa-server/releases).

It now supports three format:

- x86_64 Linux-Musl
- x86_64 Mac
- x86_64 Windows exe

## Source Code
```shell
git clone --recursive https://github.com/timzaak/spa-server
cargo build --package spa-client --release
# you could get the binary from ./target/release directory
```