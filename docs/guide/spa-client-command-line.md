# Command Line
We have provided command line in 1.x versions, but it seems no one need it. So we removed the binary release and docker release.

But you can build it from source.

## Source Code
```shell
git clone --recursive https://github.com/fornetcode/spa-server
cargo build --package spa-client --release
# you could get the binary from ./target/release directory
```
You can install it by:
```shell
cd client
cargo install --bin spa-client  --path .
```

## Overview
spa-client it a command line tool to help user upload files and release new SPA.

## Usage Example
There are some usage examples of `spa-client`, you also can get help by typing `spa-client -h`.
```shell
# upload static files to admin server, if not set $OPT_VERSION, will try to 
spa-client -c $CONFIG_PATH upload $STATIC_FILE_PATH $DOMAIN $OPT_VERSION -p 3

# tell admin server to release the specific domain version to public. if don't set $OPT_VERSION, will fetch the max version to be online, if the max version it under uploading process, release will fail. 
spa-client -c $CONFIG_PATH release $DOMAIN $OPT_VERSION

# get info of the specific domain or all domain, just like the admin server http api.
spa-client -c $CONFIG_PATH info $OPT_DOMAIN

# reload spa-server, this is used to reload https cert
spa-client -c $CONFIG_PATH reload

# delete deprecated domain files
spa-client -c $CONFIG_PATH delete $OPT_DOMAIN $OPT_MAX_RESERVE
```

There also provides http api to interact with admin server,

### Config
the config file format is hocon:

```hocon
# admin server address and auth
server {
  # required
  address: "http://127.0.0.1:9000"
  # required
  auth_token: "token"
}

# uploading file thread number.
upload {
  # optional, default value is 3.
  parallel: 3
}
```
the config file name would be `client.conf`

```shell
# Uploading Files By scp and release 
scp $SPA_DIRECTORY \
user@ip:$(curl "http://$ADMIN_SERVER/upload/position?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN") &&\
curl "http://$ADMIN_SERVER/update_version?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN"
```
