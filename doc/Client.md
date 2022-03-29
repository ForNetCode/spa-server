# SPA-Client
this doc provides all info about spa-client config and instructions for use.


## Shell Command
the config file contains information about the admin server and some default options.
its path: `config_path` default value is `./client_config.conf`, and some options could be obtained from environment variable.

```shell
# upload static files to admin server
spa-client --config-dir $CONFIG_PATH upload $DOMAIN $VERSION $STATIC_FILE_PATH  -p 3

# tell admin server to release the specific domain version to public. 
spa-client --config-dir $CONFIG_PATH release $DOMAIN $VERSION

# get info of the specific domain or all domain, just like the admin server http api.
spa-client --config-dir $CONFIG_PATH info $OPT_DOMAIN

# reload spa-server
spa-client --config-dir $CONFIG_PATH reload
```

## Config
### Config Load Order
command line option > environment  > `config-dir` file
