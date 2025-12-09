# Command Line

We provided command line client for Linux, Mac and Windows, you can download it at [Release Page](https://github.com/ForNetCode/spa-server/releases).

## Source Code

```shell
git clone --recursive https://github.com/fornetcode/spa-server
cargo build --package spa-client --release
# you could get the binary from ./target/release directory
```

Install it by:

```shell
cd client
cargo install --bin spa-client  --path .
```

## Usage Example

There are some usage examples of `spa-client`, you also can get help by typing `spa-client -h`.
The `config file path` could also set by environment variable: `SPA_CLIENT_CONFIG`.

```shell
# upload static files to admin server, if not set $OPT_VERSION, will try to 
spa-client -c $CONFIG_PATH upload $STATIC_FILE_PATH $DOMAIN $OPT_VERSION -p 3

# tell admin server to release the specific domain version to public. if don't set $OPT_VERSION, will fetch the max version to be online, if the max version it under uploading process, release will fail. 
spa-client -c $CONFIG_PATH release $DOMAIN $OPT_VERSION

# get info of the specific domain or all domain, just like the admin server http api.
spa-client -c $CONFIG_PATH info $OPT_DOMAIN


# delete deprecated domain files
spa-client -c $CONFIG_PATH delete $OPT_DOMAIN $OPT_MAX_RESERVE
```

### Config

the config file format is toml:

```toml
[server]
address = "http://127.0.0.1:9000" # SPA_SERVER_ADDRESS
auth_token = "token" # SPA_SERVER_AUTH_TOKEN
# [upload]
## default value is:3
# parallel = 3  # SPA_UPLOAD_PARALLEL
```
